use crate::agent_package::list::AzureDevOpsAgentPackageListArgs;
use crate::agent_package::show_newest::AzureDevOpsAgentPackageShowNewestArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps agent package-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPackageArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsAgentPackageCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsAgentPackageCommand {
    /// List Azure DevOps agent packages.
    List(AzureDevOpsAgentPackageListArgs),
    /// Show the newest Azure DevOps agent package (most recent `createdOn`).
    ShowNewest(AzureDevOpsAgentPackageShowNewestArgs),
}

impl AzureDevOpsAgentPackageArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsAgentPackageCommand::List(args) => {
                args.invoke(auth_context, executor).await?
            }
            AzureDevOpsAgentPackageCommand::ShowNewest(args) => {
                args.invoke(auth_context, executor).await?
            }
        }

        Ok(())
    }
}
