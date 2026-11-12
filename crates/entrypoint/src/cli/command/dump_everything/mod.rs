use crate::noninteractive::dump_everything;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Dump all collected metadata to disk.
#[derive(Facet, Debug, Clone)]
pub struct DumpEverythingArgs {
    /// Azure DevOps organization whose metadata should be exported.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl DumpEverythingArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let authentication = self.authentication.resolve(executor).await?;
        dump_everything(self.org, &authentication, executor).await?;
        Ok(())
    }
}
