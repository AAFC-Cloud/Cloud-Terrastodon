use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlementListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlementUpdateRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use tracing::info;

#[derive(Facet, Debug, Clone)]
/// Update an Azure DevOps user's license entitlement.
pub struct AzureDevOpsLicenseEntitlementUserUpdateTuiArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsLicenseEntitlementUserUpdateTuiArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;

        let chosen_entitlements = PickerTui::<_>::new()
            .set_header("Azure DevOps License Entitlements")
            .pick_many_reloadable(executor.cancellation(), async |child, invalidate| {
                let executor = &executor.with_cancellation_scope(child);
                AzureDevOpsUserLicenseEntitlementListRequest {
                    org_url: Cow::Borrowed(&self.org),
                    auth_selection,
                }
                .execute(executor)
                .with_invalidation(invalidate)
                .await
                .map(|ents| {
                    ents.into_iter()
                        .map(|e| Choice {
                            key: format!(
                                "{} <{}> ({})",
                                e.user.display_name, e.user.unique_name, e.user.id
                            ),
                            value: e,
                        })
                        .collect::<Vec<_>>()
                })
            })
            .await?;

        // Choose a single license kind
        let license = PickerTui::<_>::new()
            .set_header("Azure DevOps License Kind")
            .pick_one(AzureDevOpsLicenseType::VARIANTS, executor.cancellation())
            .await?;

        for entitlement in chosen_entitlements {
            let resp = AzureDevOpsUserLicenseEntitlementUpdateRequest {
                org_url: Cow::Borrowed(&self.org),
                auth_selection,
                user: (entitlement.user_id).into(),
                license_kind: (license.clone()),
            }
            .execute(executor)
            .await?;
            info!(
                %entitlement.user_id,
                %license,
                ?resp,
                "Successfully updated license entitlement for user"
            );
        }

        Ok(CliOutput::none())
    }
}
