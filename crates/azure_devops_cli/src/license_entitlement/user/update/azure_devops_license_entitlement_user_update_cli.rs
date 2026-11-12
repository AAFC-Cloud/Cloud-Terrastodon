use super::azure_devops_license_update_outcome::AzureDevOpsLicenseUpdateOutcome;
use super::azure_devops_license_update_report::AzureDevOpsLicenseUpdateReport;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsAccountLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseAccessLevelUpdate;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlementShowRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlementUpdateRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use tracing::error;
use tracing::info;

#[derive(Facet, Debug, Clone)]
/// Update an Azure DevOps user's license entitlement.
pub struct AzureDevOpsLicenseEntitlementUserUpdateArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        AzureDevOpsLicenseAccessLevelUpdate::try_from(&self.license)?;
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;

        let mut outcomes = Vec::new();
        for (i, user) in self.user.into_iter().enumerate() {
            let was = if let Some(ref expected) = self.has_license {
                if let Some(entitlement) = match (AzureDevOpsUserLicenseEntitlementShowRequest {
                    org_url: Cow::Borrowed(&self.org),
                    user: (&user).into(),
                    auth_selection,
                })
                .execute(executor)
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
                        outcomes.push(AzureDevOpsLicenseUpdateOutcome::SkippedDueToLicenseMatchFailureDueToMissingLicense(
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
                            outcomes.push(AzureDevOpsLicenseUpdateOutcome::NoChangeNeeded(user));
                            continue;
                        }
                        ref x if *x != *expected => {
                            error!(
                                %user,
                                existing = %x,
                                %expected,
                                "User has license that does not match expected, skipping update"
                            );
                            outcomes.push(
                                AzureDevOpsLicenseUpdateOutcome::SkippedDueToLicenseMatchFailure {
                                    user,
                                    was: entitlement,
                                },
                            );
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

            AzureDevOpsUserLicenseEntitlementUpdateRequest {
                org_url: Cow::Borrowed(&self.org),
                auth_selection,
                user: (&user).into(),
                license_kind: (self.license.clone()),
            }
            .execute(executor)
            .await?;
            outcomes.push(AzureDevOpsLicenseUpdateOutcome::Updated { user, was });
        }

        // Wait for it to take effect
        let wait = match outcomes
            .iter()
            .any(|o| matches!(o, AzureDevOpsLicenseUpdateOutcome::Updated { .. }))
        {
            true => std::time::Duration::from_mins(1),
            false => std::time::Duration::from_mins(0),
        };
        info!(
            ?wait,
            "Waiting for license entitlement updates to take effect..."
        );
        tokio::time::sleep(wait).await;

        let mut verified_outcomes = Vec::with_capacity(outcomes.len());
        let mut invalidated = false;
        for outcome in outcomes {
            let AzureDevOpsLicenseUpdateOutcome::Updated { user, was } = outcome else {
                verified_outcomes.push(outcome);
                continue;
            };
            let new_license = AzureDevOpsUserLicenseEntitlementShowRequest {
                org_url: Cow::Borrowed(&self.org),
                user: (&user).into(),
                auth_selection,
            }
            .execute(executor)
            // Refresh the shared inventory once, at the first actual verification.
            .with_invalidation(!invalidated)
            .await?;
            invalidated = true;
            if new_license.license != self.license {
                error!(
                    %user,
                    ?was,
                    expected = %self.license,
                    actual = %new_license.license,
                    "License entitlement update did not take effect after waiting"
                );
                verified_outcomes.push(AzureDevOpsLicenseUpdateOutcome::UpdateDidntTakeEffect {
                    user,
                    was,
                    now: new_license,
                });
            } else {
                verified_outcomes.push(AzureDevOpsLicenseUpdateOutcome::Updated { user, was });
            }
        }
        Ok(CliOutput::facet_with_text(
            AzureDevOpsLicenseUpdateReport {
                desired_license: self.license,
                outcomes: verified_outcomes,
            },
            AzureDevOpsLicenseUpdateReport::write_text,
        ))
    }
}
