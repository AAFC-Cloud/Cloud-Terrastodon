use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationTypeListRequest;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationTypeListArgs {
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
}

impl AzureDevOpsWorkItemRelationTypeListArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let auth_context = self.tenant.bind_auth_context(auth).await?;
        let values = AzureDevOpsWorkItemRelationTypeListRequest {
            org_url: Cow::Owned(org_url),
            auth_context: Cow::Owned(auth_context),
        }
        .await?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &values)?;
        Ok(())
    }
}
