use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlement;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;
use std::io::stdout;

/// Show a single Azure DevOps user license entitlement by user id.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserShowArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named, proxy = String)]
    pub user: AzureDevOpsUserArgument<'static>,
}

impl AzureDevOpsLicenseEntitlementUserShowArgs {
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
        let found = fetch_azure_devops_user_license_entitlement(
            &self.org,
            &self.user,
            &azure_devops_auth_context,
            executor,
        )
        .await?;
        facet_json::to_writer_std_pretty(stdout(), &found)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        Ok(())
    }
}
