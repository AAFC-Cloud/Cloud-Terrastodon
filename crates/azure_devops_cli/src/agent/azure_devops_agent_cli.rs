use crate::agent::pool::AzureDevOpsAgentPoolArgs;
use crate::agent_package::AzureDevOpsAgentPackageArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps agent-related commands (grouping for agent subcommands).
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsAgentArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsAgentCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsAgentCommand {
    /// Package-related agent commands.
    Package(AzureDevOpsAgentPackageArgs),
    /// Agent pool-related commands.
    Pool(AzureDevOpsAgentPoolArgs),
}

impl AzureDevOpsAgentArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsAgentCommand::Package(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsAgentCommand::Pool(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
