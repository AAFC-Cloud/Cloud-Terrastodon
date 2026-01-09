use crate::cli::azure_devops::project::dump::AzureDevOpsProjectDumpArgs;
use crate::cli::azure_devops::project::list::AzureDevOpsProjectListArgs;
use crate::cli::azure_devops::project::show::AzureDevOpsProjectShowArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps project-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsProjectArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsProjectCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsProjectCommand {
    /// List Azure DevOps projects in the organization.
    List(AzureDevOpsProjectListArgs),
    /// Show details for a single Azure DevOps project by id or name.
    Show(AzureDevOpsProjectShowArgs),
    /// Dump details for a single Azure DevOps project by id or name.
    Dump(AzureDevOpsProjectDumpArgs),
}

impl AzureDevOpsProjectArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsProjectCommand::List(args) => args.invoke().await?,
            AzureDevOpsProjectCommand::Show(args) => args.invoke().await?,
            AzureDevOpsProjectCommand::Dump(args) => args.invoke().await?,
        }

        Ok(())
    }
}
