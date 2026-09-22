use crate::cli::azure_devops::work_item::relation::show::AzureDevOpsWorkItemRelationShowArgs;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationChange;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationChangeRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationSelector;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationRemoveArgs {
    #[facet(flatten)]
    pub relation: AzureDevOpsWorkItemRelationShowArgs,
    /// Ask the server to validate without saving changes.
    #[facet(figue::named, default)]
    pub validate_only: bool,
    /// Do not fire any notifications for this change.
    #[facet(figue::named, default)]
    pub suppress_notifications: bool,
    /// Require this revision before changing the work item.
    #[facet(figue::named)]
    pub if_rev: Option<i32>,
}

impl AzureDevOpsWorkItemRelationRemoveArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let organization =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.relation.org)
                .await?;
        let target = self
            .relation
            .target
            .into_relation_url(&organization)
            .into_work_item()?;
        let auth_context = self.relation.tenant.bind_auth_context(auth).await?;
        let change =
            AzureDevOpsWorkItemRelationChange::Remove(AzureDevOpsWorkItemRelationSelector {
                rel: self.relation.relation_type,
                target,
            });
        let item = AzureDevOpsWorkItemRelationChangeRequest {
            org_url: Cow::Owned(organization),
            project: self.relation.project,
            auth_context: Cow::Owned(auth_context),
            id: self.relation.id,
            change,
            validate_only: self.validate_only,
            suppress_notifications: self.suppress_notifications,
            if_rev: self.if_rev,
        }
        .await?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &item)?;
        Ok(())
    }
}
