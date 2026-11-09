use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_azure_devops::update_azure_devops_user_license_entitlement;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use tracing::info;

#[derive(Facet, Debug, Clone)]
/// Update an Azure DevOps user's license entitlement.
pub struct AzureDevOpsLicenseEntitlementUserUpdateTuiArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsLicenseEntitlementUserUpdateTuiArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &azure_devops_auth_context.bind_executor(executor)?;

        let chosen_entitlements = PickerTui::<_>::new()
            .set_header("Azure DevOps License Entitlements")
            .pick_many_reloadable(async |invalidate| {
                fetch_azure_devops_user_license_entitlements(
                    &self.org,
                    &azure_devops_auth_context,
                    executor,
                )
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
            .pick_one(AzureDevOpsLicenseType::VARIANTS)
            .await?;

        for entitlement in chosen_entitlements {
            let resp = update_azure_devops_user_license_entitlement(
                &self.org,
                entitlement.user_id,
                license.clone(),
                &azure_devops_auth_context,
                executor,
            )
            .await?;
            info!(
                %entitlement.user_id,
                %license,
                ?resp,
                "Successfully updated license entitlement for user"
            );
        }

        Ok(())
    }
}
