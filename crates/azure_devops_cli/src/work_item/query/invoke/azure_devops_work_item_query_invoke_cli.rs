use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryInvokeRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQuerySelector;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemQueryInvokeArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project ID or name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
    /// Saved query UUID. Supply either this or --wiql.
    #[facet(figue::positional)]
    pub id: Option<String>,
    /// WIQL text, or @file containing WIQL.
    #[facet(figue::named)]
    pub wiql: Option<String>,
}

impl AzureDevOpsWorkItemQueryInvokeArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let query = match (self.id, self.wiql) {
            (Some(id), None) => AzureDevOpsWorkItemQuerySelector::Id(id.parse()?),
            (None, Some(wiql)) => AzureDevOpsWorkItemQuerySelector::Wiql(
                cloud_terrastodon_rest::read_optional_body(Some(wiql))
                    .await?
                    .ok_or_else(|| eyre::eyre!("Missing text input"))?,
            ),
            _ => {
                bail!("Specify exactly one saved query ID or --wiql");
            }
        };
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let result = AzureDevOpsWorkItemQueryInvokeRequest {
            org_url: Cow::Owned(self.org),
            project: Some(self.project),
            auth_selection,
            query,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(result))
    }
}
