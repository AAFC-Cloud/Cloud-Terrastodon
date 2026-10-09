use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemFieldDefinitionListRequest;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemFieldDefinitionListArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
}

impl AzureDevOpsWorkItemFieldDefinitionListArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let auth_context = self.tenant.bind_auth_context(auth).await?;
        let values = AzureDevOpsWorkItemFieldDefinitionListRequest {
            org_url: Cow::Owned(self.org),
            auth_context: Cow::Owned(auth_context),
        }
        .await?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &values)?;
        Ok(())
    }
}
