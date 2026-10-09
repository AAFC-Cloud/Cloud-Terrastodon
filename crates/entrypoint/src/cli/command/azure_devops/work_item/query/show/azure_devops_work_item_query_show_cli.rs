use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryGetRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryPath;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemQueryShowArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Saved query UUID or folder/query path.
    #[facet(figue::positional)]
    pub query: AzureDevOpsWorkItemQueryPath,
}

impl AzureDevOpsWorkItemQueryShowArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let auth_context = self.tenant.bind_auth_context(auth).await?;
        let query = AzureDevOpsWorkItemQueryGetRequest {
            org_url: Cow::Owned(self.org),
            project: self.project,
            auth_context: Cow::Owned(auth_context),
            query: self.query,
        }
        .await?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &query)?;
        Ok(())
    }
}
