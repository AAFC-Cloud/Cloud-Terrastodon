use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::fetch_azure_devops_agent_pools;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps agent pools in the organization.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPoolListArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    /// Include hosted pools.
    #[facet(figue::named)]
    pub all: bool,
}

impl AzureDevOpsAgentPoolListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let pools = fetch_azure_devops_agent_pools(&org_url).await?;
        let pools: Vec<_> = if self.all {
            pools
        } else {
            pools.into_iter().filter(|pool| !pool.is_hosted).collect()
        };

        to_writer_pretty(stdout(), &pools)?;
        Ok(())
    }
}
