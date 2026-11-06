use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsAccountLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseAccessLevelUpdate;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlement;
use cloud_terrastodon_azure_devops::update_azure_devops_user_license_entitlement;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use color_eyre::owo_colors::OwoColorize;
use eyre::Result;
use std::sync::Arc;
use tracing::error;
use tracing::info;

#[derive(facet::Facet, Debug, Clone)]
/// Update an Azure DevOps user's license entitlement.
pub struct AzureDevOpsLicenseEntitlementUserUpdateArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named)]
    pub user: Vec<AzureDevOpsUserArgument<'static>>,

    /// Bail if the user has a license that does not equal this field.
    /// Bail if the user has no license when this field is not Stakeholder.
    #[facet(figue::named)]
    pub has_license: Option<AzureDevOpsLicenseType>,

    /// Desired license kind (e.g. "Account-Express", "Account-Advanced")
    #[facet(figue::named)]
    pub license: AzureDevOpsLicenseType,

    /// Don't use cached license entitlement information, fetch fresh from Azure DevOps.
    #[facet(
        figue::named,
        figue::alias = "skip-cache",
        figue::alias = "clean",
        default = false
    )]
    pub no_cache: bool,
}

impl AzureDevOpsLicenseEntitlementUserUpdateArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        AzureDevOpsLicenseAccessLevelUpdate::try_from(&self.license)?;
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &azure_devops_auth_context.bind_executor(executor)?;

        #[derive(Default)]
        enum Outcome {
            NoChangeNeeded(AzureDevOpsUserArgument<'static>),
            SkippedDueToLicenseMatchFailureDueToMissingLicense(AzureDevOpsUserArgument<'static>),
            SkippedDueToLicenseMatchFailure {
                user: AzureDevOpsUserArgument<'static>,
                was: Arc<AzureDevOpsUserLicenseEntitlement>,
            },
            Updated {
                user: AzureDevOpsUserArgument<'static>,
                was: Option<Arc<AzureDevOpsUserLicenseEntitlement>>,
            },
            UpdateDidntTakeEffect {
                user: AzureDevOpsUserArgument<'static>,
                was: Option<Arc<AzureDevOpsUserLicenseEntitlement>>,
                now: Arc<AzureDevOpsUserLicenseEntitlement>,
            },
            #[default]
            None,
        }
        let mut outcomes = Vec::new();
        for (i, user) in self.user.into_iter().enumerate() {
            let was = if let Some(ref expected) = self.has_license {
                if let Some(entitlement) = match fetch_azure_devops_user_license_entitlement(
                    &self.org,
                    &user,
                    &azure_devops_auth_context,
                    executor,
                )
                .with_invalidation(i == 0 && self.no_cache)
                .await
                {
                    Ok(entitlement) => Some(entitlement),
                    Err(_)
                        if expected
                            == &AzureDevOpsLicenseType::Account(
                                AzureDevOpsAccountLicenseType::Stakeholder,
                            ) =>
                    {
                        None
                    }
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

            update_azure_devops_user_license_entitlement(
                &self.org,
                &user,
                self.license.clone(),
                &azure_devops_auth_context,
                executor,
            )
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
            let new_license = fetch_azure_devops_user_license_entitlement(
                &self.org,
                &user,
                &azure_devops_auth_context,
                executor,
            )
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
        for outcome in &outcomes {
            let (user, was, now) = match outcome {
                Outcome::NoChangeNeeded(user) => (user, Some(&self.license), Some(&self.license)),
                Outcome::SkippedDueToLicenseMatchFailureDueToMissingLicense(user) => {
                    (user, None, None)
                }
                Outcome::SkippedDueToLicenseMatchFailure {
                    user,
                    was: entitlement,
                } => (user, Some(&entitlement.license), Some(&entitlement.license)),
                Outcome::Updated { user, was } => {
                    (user, was.as_ref().map(|x| &x.license), Some(&self.license))
                }
                Outcome::UpdateDidntTakeEffect { user, was, now } => {
                    (user, was.as_ref().map(|x| &x.license), Some(&now.license))
                }
                Outcome::None => unreachable!(),
            };
            print!("{:64} was ", user.cyan());
            match was {
                Some(x) if x == &self.license => print!("{} (as desired)", x.green()),
                Some(x) => print!("{}", x.yellow()),
                None => print!("{}", "None".yellow()),
            };
            print!(" now ");
            match now {
                Some(x) if x == &self.license => print!("{} (as desired)", x.green()),
                Some(x) => print!("{}", x.red()),
                None => print!("{}", "None".red()),
            };
            println!();
        }

        Ok(())
    }
}
