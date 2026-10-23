use crate::agent::pool::AzureDevOpsAgentPoolArgs;
use crate::agent_package::AzureDevOpsAgentPackageArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsAgentCommand::Package(args) => args.invoke(cancellation).await?,
            AzureDevOpsAgentCommand::Pool(args) => args.invoke(auth_context, cancellation).await?,
        }

        Ok(())
    }
}
