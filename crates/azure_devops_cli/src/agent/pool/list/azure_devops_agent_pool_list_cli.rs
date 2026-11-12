use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

/// List Azure DevOps agent pools in the organization.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPoolListArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Include hosted pools.
    #[facet(figue::named)]
    pub all: bool,
}

impl AzureDevOpsAgentPoolListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let pools = AzureDevOpsAgentPoolListRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
        }
        .execute(executor)
        .await?;
        let pools: Vec<_> = pools
            .iter()
            .filter(|pool| self.all || !pool.is_hosted)
            .cloned()
            .collect();

        Ok(CliOutput::facet(pools))
    }
}
