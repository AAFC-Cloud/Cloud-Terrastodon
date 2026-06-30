use crate::cli::azure_devops::agent::pool::AzureDevOpsAgentPoolArgs;
use crate::cli::azure_devops::agent_package::AzureDevOpsAgentPackageArgs;
use eyre::Result;

/// Azure DevOps agent-related commands (grouping for agent subcommands).
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsAgentArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsAgentCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsAgentCommand {
    /// Package-related agent commands.
    Package(AzureDevOpsAgentPackageArgs),
    /// Agent pool-related commands.
    Pool(AzureDevOpsAgentPoolArgs),
}

impl AzureDevOpsAgentArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsAgentCommand::Package(args) => args.invoke().await?,
            AzureDevOpsAgentCommand::Pool(args) => args.invoke().await?,
        }

        Ok(())
    }
}
