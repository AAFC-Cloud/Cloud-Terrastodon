use crate::agent::pool::AzureDevOpsAgentPoolArgs;
use crate::agent_package::AzureDevOpsAgentPackageArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
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
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsAgentCommand::Package(args) => args.invoke(auth_context, executor).await?,
            AzureDevOpsAgentCommand::Pool(args) => args.invoke(auth_context, executor).await?,
        }

        Ok(())
    }
}
