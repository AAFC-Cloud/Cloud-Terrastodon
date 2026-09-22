use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryCreateRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryPath;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemQueryCreateArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    /// Project id or project name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    /// Tenant id or tracked alias for delegated authentication.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
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
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let auth_context = self.tenant.bind_auth_context(auth).await?;
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let query = AzureDevOpsWorkItemQueryCreateRequest {
            org_url: Cow::Borrowed(&org_url),
            project: self.project,
            auth_context: Cow::Borrowed(&auth_context),
            folder: self.folder,
            name: self.name,
            wiql: cloud_terrastodon_rest::read_optional_body(Some(self.wiql))
                .await?
                .ok_or_else(|| eyre::eyre!("Missing text input"))?,
        }
        .await?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &query)?;
        Ok(())
    }
}
