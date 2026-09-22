use crate::cli::azure_devops::work_item::field::show::AzureDevOpsWorkItemFieldShowArgs;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure_devops::AzureDevOpsJsonPatch;
use cloud_terrastodon_azure_devops::AzureDevOpsJsonPatchOperation;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemUpdateRequest;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemFieldSetArgs {
    #[facet(flatten)]
    pub field: AzureDevOpsWorkItemFieldShowArgs,
    /// JSON value (quote JSON strings), or @file.
    #[facet(figue::named)]
    pub value: String,
    /// Ask the server to validate without saving changes.
    #[facet(figue::named, default)]
    pub validate_only: bool,
    /// Do not fire any notifications for this change.
    #[facet(figue::named, default)]
    pub suppress_notifications: bool,
    /// Require this revision before updating the work item.
    #[facet(figue::named)]
    pub if_rev: Option<i32>,
}

impl AzureDevOpsWorkItemFieldSetArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let patch = AzureDevOpsJsonPatch::try_from(vec![AzureDevOpsJsonPatchOperation::Add {
            path: format!(
                "/fields/{}",
                self.field.field.replace('~', "~0").replace('/', "~1")
            ),
            value: {
                let body = cloud_terrastodon_rest::read_optional_body(Some(self.value))
                    .await?
                    .ok_or_else(|| eyre::eyre!("Missing JSON input"))?;
                facet_json::from_str(&body)
                    .map_err(|error| eyre::eyre!("Invalid JSON input: {error}"))?
            },
        }])?;
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.field.org).await?;
        let auth_context = self.field.tenant.bind_auth_context(auth).await?;
        let item = AzureDevOpsWorkItemUpdateRequest {
            org_url: Cow::Owned(org_url),
            project: self.field.project,
            auth_context: Cow::Owned(auth_context),
            id: self.field.id,
            patch,
            validate_only: self.validate_only,
            suppress_notifications: self.suppress_notifications,
            if_rev: self.if_rev,
        }
        .await?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &item)?;
        Ok(())
    }
}
