use crate::agent::pool::entitlement::AzureDevOpsAgentPoolEntitlementArgs;
use crate::agent::pool::list::AzureDevOpsAgentPoolListArgs;
use crate::agent::pool::summary::AzureDevOpsAgentPoolSummaryArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps agent pool-related commands (grouping for pool subcommands).
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPoolArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsAgentPoolCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsAgentPoolCommand {
    /// List Azure DevOps agent pools in the organization.
    List(AzureDevOpsAgentPoolListArgs),
    /// Agent pool entitlement-related commands.
    Entitlement(AzureDevOpsAgentPoolEntitlementArgs),
    /// Summary of agent pools and projects that use them.
    Summary(AzureDevOpsAgentPoolSummaryArgs),
}

impl AzureDevOpsAgentPoolArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsAgentPoolCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsAgentPoolCommand::Entitlement(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsAgentPoolCommand::Summary(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
