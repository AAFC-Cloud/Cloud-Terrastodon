use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryCreateRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryPath;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemQueryCreateArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
    /// Existing destination folder UUID or path.
    #[facet(figue::named)]
    pub folder: AzureDevOpsWorkItemQueryPath,
    /// Name of the saved query.
    #[facet(figue::named)]
    pub name: String,
    /// WIQL text, or @file containing WIQL.
    #[facet(figue::named)]
    pub wiql: String,
}

impl AzureDevOpsWorkItemQueryCreateArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let query = AzureDevOpsWorkItemQueryCreateRequest {
            org_url: Cow::Borrowed(&self.org),
            project: self.project,
            auth_selection,
            folder: self.folder,
            name: self.name,
            wiql: cloud_terrastodon_rest::read_optional_body(Some(self.wiql))
                .await?
                .ok_or_else(|| eyre::eyre!("Missing text input"))?,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(query))
    }
}
