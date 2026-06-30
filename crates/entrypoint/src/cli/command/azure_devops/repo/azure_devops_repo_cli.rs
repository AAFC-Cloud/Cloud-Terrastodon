use crate::cli::azure_devops::repo::list::AzureDevOpsRepoListArgs;
use crate::cli::azure_devops::repo::show::AzureDevOpsRepoShowArgs;
use eyre::Result;

/// Azure DevOps repository-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsRepoArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsRepoCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsRepoCommand {
    /// List Azure DevOps repos in the project.
    List(AzureDevOpsRepoListArgs),
    /// Show details for a single Azure DevOps repo.
    Show(AzureDevOpsRepoShowArgs),
}

impl AzureDevOpsRepoArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsRepoCommand::List(args) => args.invoke().await?,
            AzureDevOpsRepoCommand::Show(args) => args.invoke().await?,
        }

        Ok(())
    }
}
