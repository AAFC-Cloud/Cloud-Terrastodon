use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemExpand;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemsGetRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemListArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project ID or name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
    /// Comma-separated work item IDs. No other items are queried.
    #[facet(figue::named)]
    pub ids: String,
    /// Response expansion (all includes fields and relations).
    #[facet(figue::named, default)]
    pub expand: AzureDevOpsWorkItemExpand,
    /// Comma-separated field reference names.
    #[facet(figue::named)]
    pub fields: Option<String>,
    /// Read a historical snapshot at this UTC timestamp.
    #[facet(figue::named)]
    pub as_of: Option<DateTime<Utc>>,
}

impl AzureDevOpsWorkItemListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let ids = self
            .ids
            .split(',')
            .map(|id| id.trim().parse())
            .collect::<Result<Vec<_>>>()?;
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let items = AzureDevOpsWorkItemsGetRequest {
            org_url: Cow::Owned(self.org),
            project: Some(self.project),
            auth_selection,
            ids,
            expand: self.expand,
            fields: self
                .fields
                .map(|fields| {
                    fields
                        .split(',')
                        .map(|field| field.trim().to_owned())
                        .collect()
                })
                .unwrap_or_default(),
            as_of: self.as_of,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(items))
    }
}
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
