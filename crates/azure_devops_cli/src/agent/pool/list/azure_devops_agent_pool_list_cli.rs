use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::fetch_azure_devops_agent_pools;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps agent pools in the organization.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPoolListArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Include hosted pools.
    #[facet(figue::named)]
    pub all: bool,
}

impl AzureDevOpsAgentPoolListArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &azure_devops_auth_context.bind_executor(executor)?;
        let pools =
            fetch_azure_devops_agent_pools(&self.org, &azure_devops_auth_context, executor).await?;
        let pools: Vec<_> = pools
            .iter()
            .filter(|pool| self.all || !pool.is_hosted)
            .collect();

        facet_json::to_writer_std_pretty(stdout(), &pools)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        Ok(())
    }
}
