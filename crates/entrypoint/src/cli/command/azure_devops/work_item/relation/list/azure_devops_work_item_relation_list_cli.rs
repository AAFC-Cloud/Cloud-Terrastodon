use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemGetRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationTypeName;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationListArgs {
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    #[facet(figue::named)]
    pub project: Option<AzureDevOpsProjectArgument<'static>>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Work item ID.
    #[facet(figue::positional)]
    pub id: AzureDevOpsWorkItemId,
    /// Optional relation reference name filter.
    #[facet(figue::named, rename = "type")]
    pub relation_type: Option<AzureDevOpsWorkItemRelationTypeName>,
}

impl AzureDevOpsWorkItemRelationListArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let auth_context = self.tenant.bind_auth_context(auth).await?;
        let relations: Vec<_> = AzureDevOpsWorkItemGetRequest {
            org_url: Cow::Owned(org_url),
            project: self.project,
            auth_context: Cow::Owned(auth_context),
            id: self.id,
            expand: Default::default(),
            fields: Vec::new(),
            as_of: None,
        }
        .await?
        .relations
        .unwrap_or_default()
        .into_iter()
        .filter(|r| {
            self.relation_type
                .as_ref()
                .is_none_or(|kind| &r.rel == kind)
        })
        .collect();
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &relations)?;
        Ok(())
    }
}
