use crate::noninteractive::dump_azure_devops;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Dump Azure DevOps project, user, and group metadata.
#[derive(Facet, Debug, Clone)]
pub struct DumpAzureDevOpsArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl DumpAzureDevOpsArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(&self.org, executor).await?;
        dump_azure_devops(self.org, authentication, executor).await
    }
}
