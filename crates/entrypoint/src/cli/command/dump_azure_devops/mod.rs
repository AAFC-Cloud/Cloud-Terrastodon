use crate::noninteractive::dump_azure_devops;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;

/// Dump Azure DevOps metadata to disk.
#[derive(facet::Facet, Debug, Clone)]
pub struct DumpAzureDevOpsArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl DumpAzureDevOpsArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        dump_azure_devops(self.org, &auth_context, cancellation).await?;
        Ok(())
    }
}
