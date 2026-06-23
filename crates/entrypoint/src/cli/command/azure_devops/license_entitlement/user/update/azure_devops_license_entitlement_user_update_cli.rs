use clap::Args;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlement;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_azure_devops::update_azure_devops_user_license_entitlement;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use color_eyre::owo_colors::OwoColorize;
use eyre::Result;
use eyre::bail;
use tracing::error;
use tracing::info;

#[derive(Args, Debug, Clone)]
/// Update an Azure DevOps user's license entitlement.
pub struct AzureDevOpsLicenseEntitlementUserUpdateArgs {
    #[arg(long)]
    pub user: Vec<AzureDevOpsUserArgument<'static>>,

    /// Bail if the user has a license that does not equal this field.
    /// Bail if the user has no license when this field is not Stakeholder.
    #[arg(long)]
    pub has_license: Option<AzureDevOpsLicenseType>,

    /// Desired license kind (e.g. "Account-Express", "Account-Advanced")
    #[arg(long)]
    pub license: AzureDevOpsLicenseType,

    /// Don't use cached license entitlement information, fetch fresh from Azure DevOps.
    #[arg(long, default_value_t = false)]
    pub no_cache: bool,
}

impl AzureDevOpsLicenseEntitlementUserUpdateArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;

        if let AzureDevOpsLicenseType::Other(s) = &self.license {
            bail!("Invalid license kind specified: {}", s);
        };

        #[derive(Default)]
        enum Outcome {
            NoChangeNeeded(AzureDevOpsUserArgument<'static>),
            SkippedDueToLicenseMatchFailureDueToMissingLicense(AzureDevOpsUserArgument<'static>),
            SkippedDueToLicenseMatchFailure {
                user: AzureDevOpsUserArgument<'static>,
                was: AzureDevOpsUserLicenseEntitlement,
            },
            Updated {
                user: AzureDevOpsUserArgument<'static>,
                was: Option<AzureDevOpsUserLicenseEntitlement>,
            },
            UpdateDidntTakeEffect {
                user: AzureDevOpsUserArgument<'static>,
                was: Option<AzureDevOpsUserLicenseEntitlement>,
                now: AzureDevOpsUserLicenseEntitlement,
            },
            #[default]
            None,
        }
        let mut outcomes = Vec::new();
        for (i, user) in self.user.into_iter().enumerate() {
            let was = if let Some(ref expected) = self.has_license {
                if let Some(entitlement) = match fetch_azure_devops_user_license_entitlement(
                    &org_url, &user,
                )
                .with_invalidation(i == 0 && self.no_cache)
                .await
                {
                    Ok(entitlement) => Some(entitlement),
                    Err(_) if expected == &AzureDevOpsLicenseType::AccountStakeholder => None,
                    Err(e) => {
                        error!(
                            %user,
                            %expected,
                            "Failed to fetch license entitlement for user, skipping due to expected license requirement: {e:#}"
                        );
                        outcomes.push(Outcome::SkippedDueToLicenseMatchFailureDueToMissingLicense(
                            user,
                        ));
                        continue;
                    }
                } {
                    match entitlement.license {
                        ref x if *x == self.license => {
                            info!(
                                %user,
                                license = %x,
                                "User already has desired license, no change needed"
                            );
                            outcomes.push(Outcome::NoChangeNeeded(user));
                            continue;
                        }
                        ref x if *x != *expected => {
                            error!(
                                %user,
                                existing = %x,
                                %expected,
                                "User has license that does not match expected, skipping update"
                            );
                            outcomes.push(Outcome::SkippedDueToLicenseMatchFailure {
                                user,
                                was: entitlement,
                            });
                            continue;
                        }
                        ref x if *x == *expected => Some(entitlement),
                        _ => unreachable!(),
                    }
                } else {
                    None
                }
            } else {
                None
            };

            info!(
                ?user,
                ?self.has_license,
                %self.license,
                "Updating license entitlement for user"
            );

            update_azure_devops_user_license_entitlement(&org_url, &user, self.license.clone())
                .await?;
            outcomes.push(Outcome::Updated { user, was });
        }

        // Wait for it to take effect
        let wait = match outcomes
            .iter()
            .any(|o| matches!(o, Outcome::Updated { .. }))
        {
            true => std::time::Duration::from_mins(1),
            false => std::time::Duration::from_mins(0),
        };
        info!(
            ?wait,
            "Waiting for license entitlement updates to take effect..."
        );
        tokio::time::sleep(wait).await;

        for (i, outcome) in outcomes.iter_mut().enumerate() {
            let Outcome::Updated { .. } = outcome else {
                continue;
            };
            let Outcome::Updated { user, was } = std::mem::take(outcome) else {
                unreachable!()
            };

            // Fetch to verify
            let new_license = fetch_azure_devops_user_license_entitlement(&org_url, &user)
                // underlying cache is shared for all users, so only invalidate on the first check
                .with_invalidation(i == 0)
                .await?;

            if new_license.license != self.license {
                error!(
                    %user,
                    ?was,
                    expected = %self.license,
                    actual = %new_license.license,
                    "License entitlement update did not take effect after waiting"
                );
                *outcome = Outcome::UpdateDidntTakeEffect {
                    user: user.clone(),
                    was,
                    now: new_license,
                };
            }
        }

        // pretty print a final report
        for outcome in outcomes {
            let (user, was, now) = match outcome {
                Outcome::NoChangeNeeded(user) => {
                    (user, Some(self.license.clone()), Some(self.license.clone()))
                }
                Outcome::SkippedDueToLicenseMatchFailureDueToMissingLicense(user) => {
                    (user, None, None)
                }
                Outcome::SkippedDueToLicenseMatchFailure {
                    user,
                    was: entitlement,
                } => (
                    user,
                    Some(entitlement.license.clone()),
                    Some(entitlement.license),
                ),
                Outcome::Updated { user, was } => {
                    (user, was.map(|x| x.license), Some(self.license.clone()))
                }
                Outcome::UpdateDidntTakeEffect { user, was, now } => {
                    (user, was.map(|x| x.license), Some(now.license))
                }
                Outcome::None => unreachable!(),
            };
            print!("{:64} was ", user.cyan());
            match was {
                Some(x) if x == self.license => print!("{} (as desired)", x.green()),
                Some(x) => print!("{}", x.yellow()),
                None => print!("{}", "None".yellow()),
            };
            print!(" now ");
            match now {
                Some(x) if x == self.license => print!("{} (as desired)", x.green()),
                Some(x) => print!("{}", x.red()),
                None => print!("{}", "None".red()),
            };
            println!();
        }

        Ok(())
    }
}
