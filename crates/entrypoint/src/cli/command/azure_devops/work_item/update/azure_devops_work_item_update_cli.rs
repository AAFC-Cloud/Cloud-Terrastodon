use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemUpdateRequest;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemUpdateArgs {
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    #[facet(figue::named)]
    pub project: Option<AzureDevOpsProjectArgument<'static>>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Work item ID.
    #[facet(figue::positional)]
    pub id: AzureDevOpsWorkItemId,
    /// JSON Patch array, or @file.
    #[facet(figue::named)]
    pub patch: String,
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

impl AzureDevOpsWorkItemUpdateArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let body = cloud_terrastodon_rest::read_optional_body(Some(self.patch))
            .await?
            .ok_or_else(|| eyre::eyre!("Missing JSON input"))?;
        let patch = facet_json::from_str(&body)
            .map_err(|error| eyre::eyre!("Invalid JSON input: {error}"))?;
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let auth_context = self.tenant.bind_auth_context(auth).await?;
        let item = AzureDevOpsWorkItemUpdateRequest {
            org_url: Cow::Owned(org_url),
            project: self.project,
            auth_context: Cow::Owned(auth_context),
            id: self.id,
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
