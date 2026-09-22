use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryListRequest;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemQueryListArgs {
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Folder expansion depth, from zero to two.
    #[facet(figue::named, default = 2)]
    pub depth: u32,
}

impl AzureDevOpsWorkItemQueryListArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let auth_context = self.tenant.bind_auth_context(auth).await?;
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let queries = AzureDevOpsWorkItemQueryListRequest {
            org_url: Cow::Owned(org_url),
            project: self.project,
            auth_context: Cow::Owned(auth_context),
            depth: self.depth,
        }
        .await?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &queries)?;
        Ok(())
    }
}
