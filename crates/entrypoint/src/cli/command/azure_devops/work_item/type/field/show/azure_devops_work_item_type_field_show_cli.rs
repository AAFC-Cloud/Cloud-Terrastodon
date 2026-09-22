use crate::cli::azure_devops::work_item::r#type::field::list::AzureDevOpsWorkItemTypeFieldListArgs;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemFieldName;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemTypeFieldGetRequest;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemTypeFieldShowArgs {
    #[facet(flatten)]
    pub args: AzureDevOpsWorkItemTypeFieldListArgs,
    /// Field reference name.
    #[facet(figue::positional)]
    pub field: AzureDevOpsWorkItemFieldName,
}

impl AzureDevOpsWorkItemTypeFieldShowArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.args.org).await?;
        let auth_context = self.args.tenant.bind_auth_context(auth).await?;
        let value = AzureDevOpsWorkItemTypeFieldGetRequest {
            org_url: Cow::Owned(org_url),
            project: self.args.project,
            auth_context: Cow::Owned(auth_context),
            work_item_type: self.args.work_item_type,
            field: self.field,
        }
        .await?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &value)?;
        Ok(())
    }
}
