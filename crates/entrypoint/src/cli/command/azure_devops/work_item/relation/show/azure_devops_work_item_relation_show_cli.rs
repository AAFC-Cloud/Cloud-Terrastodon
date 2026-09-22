use crate::cli::azure_devops::work_item::relation::AzureDevOpsWorkItemRelationTarget;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemGetRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationSelector;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationTypeName;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationShowArgs {
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    #[facet(figue::named)]
    pub project: Option<AzureDevOpsProjectArgument<'static>>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Work item ID.
    #[facet(figue::positional)]
    pub id: AzureDevOpsWorkItemId,
    /// Relation reference name, such as System.LinkTypes.Related.
    #[facet(figue::named, rename = "type")]
    pub relation_type: AzureDevOpsWorkItemRelationTypeName,
    /// Target work item ID or relation URL. The target is not fetched.
    #[facet(figue::named)]
    pub target: AzureDevOpsWorkItemRelationTarget,
}

impl AzureDevOpsWorkItemRelationShowArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let organization =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let target = self
            .target
            .into_relation_url(&organization)
            .into_work_item()?;
        let project = self.project;
        let auth_context = self.tenant.bind_auth_context(auth).await?;
        let item = AzureDevOpsWorkItemGetRequest {
            org_url: Cow::Owned(organization.clone()),
            project,
            auth_context: Cow::Owned(auth_context),
            id: self.id,
            expand: Default::default(),
            fields: Vec::new(),
            as_of: None,
        }
        .await?;
        let index = AzureDevOpsWorkItemRelationSelector {
            rel: self.relation_type,
            target,
        }
        .index(&organization, &item)?;
        let relation = &item
            .relations
            .as_ref()
            .ok_or_else(|| eyre::eyre!("No relations returned"))?[index];
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), relation)?;
        Ok(())
    }
}
