use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionGetRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

/// Show a pipeline's full definition, including repository and YAML process details.
///
/// See Microsoft's [Definitions Get 7.1 API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1).
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsBuildDefinitionShowArgs {
    /// Build definition ID from `build definition list`.
    #[facet(figue::positional, figue::label = "ID")]
    pub id: AzureDevOpsBuildDefinitionId,
    /// Organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project ID or name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
    /// Retrieve a specific definition revision. Defaults to the latest revision.
    #[facet(figue::named)]
    pub revision: Option<i32>,
    /// Refresh cached results for this definition before fetching it.
    #[facet(
        figue::named,
        figue::alias = "skip-cache",
        figue::alias = "clean",
        default = false
    )]
    pub no_cache: bool,
}

impl AzureDevOpsBuildDefinitionShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let definition = AzureDevOpsBuildDefinitionGetRequest {
            org_url: Cow::Borrowed(&self.org),
            project: (self.project),
            auth_selection,
            definition_id: (self.id),
            revision: (self.revision),
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        Ok(CliOutput::facet(definition))
    }
}
