use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemFieldDefinitionListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemFieldDefinitionListArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
}

impl AzureDevOpsWorkItemFieldDefinitionListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let values = AzureDevOpsWorkItemFieldDefinitionListRequest {
            org_url: Cow::Owned(self.org),
            auth_selection,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(values))
    }
}
