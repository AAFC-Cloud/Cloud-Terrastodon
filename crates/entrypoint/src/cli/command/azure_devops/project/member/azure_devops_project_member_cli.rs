use crate::cli::azure_devops::project::member::list::AzureDevOpsProjectMemberListArgs;
use eyre::Result;

/// Azure DevOps project member-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsProjectMemberArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsProjectMemberCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
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
