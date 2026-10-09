use crate::noninteractive::dump_azure_devops;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Dump Azure DevOps metadata to disk.
#[derive(facet::Facet, Debug, Clone)]
pub struct DumpAzureDevOpsArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl DumpAzureDevOpsArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        dump_azure_devops(self.org, auth_context).await?;
        Ok(())
    }
}
