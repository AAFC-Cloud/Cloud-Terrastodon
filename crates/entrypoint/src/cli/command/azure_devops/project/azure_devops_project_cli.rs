use crate::cli::azure_devops::project::dump::AzureDevOpsProjectDumpArgs;
use crate::cli::azure_devops::project::list::AzureDevOpsProjectListArgs;
use crate::cli::azure_devops::project::member::AzureDevOpsProjectMemberArgs;
use crate::cli::azure_devops::project::show::AzureDevOpsProjectShowArgs;
use eyre::Result;

/// Azure DevOps project-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsProjectArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsProjectCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsProjectCommand {
    /// List Azure DevOps projects in the organization.
    List(AzureDevOpsProjectListArgs),
    /// Show details for a single Azure DevOps project by id or name.
    Show(AzureDevOpsProjectShowArgs),
    /// Dump details for a single Azure DevOps project by id or name.
    Dump(AzureDevOpsProjectDumpArgs),
    /// Project member operations.
    Member(AzureDevOpsProjectMemberArgs),
}

impl AzureDevOpsProjectArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsProjectCommand::List(args) => args.invoke().await?,
            AzureDevOpsProjectCommand::Show(args) => args.invoke().await?,
            AzureDevOpsProjectCommand::Dump(args) => args.invoke().await?,
            AzureDevOpsProjectCommand::Member(args) => args.invoke().await?,
        }

        Ok(())
    }
}
