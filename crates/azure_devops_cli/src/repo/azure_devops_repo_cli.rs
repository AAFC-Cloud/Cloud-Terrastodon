use crate::repo::list::AzureDevOpsRepoListArgs;
use crate::repo::show::AzureDevOpsRepoShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps repository-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsRepoArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsRepoCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsRepoCommand {
    /// List Azure DevOps repos in the project.
    List(AzureDevOpsRepoListArgs),
    /// Show details for a single Azure DevOps repo.
    Show(AzureDevOpsRepoShowArgs),
}

impl AzureDevOpsRepoArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsRepoCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsRepoCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
