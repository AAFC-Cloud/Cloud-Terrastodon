use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemUpdateRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemUpdateArgs {
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let body = cloud_terrastodon_rest::read_optional_body(Some(self.patch))
            .await?
            .ok_or_else(|| eyre::eyre!("Missing JSON input"))?;
        let patch = facet_json::from_str(&body)
            .map_err(|error| eyre::eyre!("Invalid JSON input: {error}"))?;
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let item = AzureDevOpsWorkItemUpdateRequest {
            org_url: Cow::Owned(self.org),
            project: Some(self.project),
            auth_selection,
            id: self.id,
            patch,
            validate_only: self.validate_only,
            suppress_notifications: self.suppress_notifications,
            if_rev: self.if_rev,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(item))
    }
}
