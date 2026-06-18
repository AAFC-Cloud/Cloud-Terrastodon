use crate::cli::azure_devops::project::member::list::AzureDevOpsProjectMemberListArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps project member-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsProjectMemberArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsProjectMemberCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsProjectMemberCommand {
    /// List users that are transitively members of the project.
    List(AzureDevOpsProjectMemberListArgs),
}

impl AzureDevOpsProjectMemberArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsProjectMemberCommand::List(args) => args.invoke().await?,
        }

        Ok(())
    }
}
