use crate::work_item::relation::AzureDevOpsWorkItemRelationTarget;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemGetRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationSelector;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationTypeName;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemRelationShowArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project ID or name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let target = self.target.into_relation_url(&self.org).into_work_item()?;
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let item = AzureDevOpsWorkItemGetRequest {
            org_url: Cow::Borrowed(&self.org),
            project: Some(self.project),
            auth_selection,
            id: self.id,
            expand: Default::default(),
            fields: Vec::new(),
            as_of: None,
        }
        .execute(executor)
        .await?;
        let index = AzureDevOpsWorkItemRelationSelector {
            rel: self.relation_type,
            target,
        }
        .index(&self.org, &item)?;
        let relation = &item
            .relations
            .as_ref()
            .ok_or_else(|| eyre::eyre!("No relations returned"))?[index];
        Ok(CliOutput::facet(relation.clone()))
    }
}
