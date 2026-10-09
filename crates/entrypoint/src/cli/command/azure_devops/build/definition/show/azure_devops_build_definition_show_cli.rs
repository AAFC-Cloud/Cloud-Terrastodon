use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_build_definition;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Show a pipeline's full definition, including repository and YAML process details.
///
/// See Microsoft's [Definitions Get 7.1 API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1).
#[derive(Debug, Clone, facet::Facet)]
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
    /// Tenant ID or tracked alias for delegated authentication.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        let auth_context = self.tenant.bind_auth_context(auth_context).await?;
        let definition = fetch_azure_devops_build_definition(
            &self.org,
            self.project,
            &auth_context,
            self.id,
            self.revision,
        )
        .with_invalidation(self.no_cache)
        .await?;
        Ok(CliOutput::facet(definition))
    }
}
