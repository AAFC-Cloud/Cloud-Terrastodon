use cloud_terrastodon_app::CliOutput;
use facet::Facet;
pub mod gitea_repo_browse_cli;
pub mod gitea_repo_list_cli;
pub mod gitea_repo_show_cli;

use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
pub use gitea_repo_browse_cli::GiteaRepoBrowseArgs;
pub use gitea_repo_list_cli::GiteaRepoListArgs;
pub use gitea_repo_show_cli::GiteaRepoShowArgs;

#[derive(Facet, Debug, Clone)]
pub struct GiteaRepoArgs {
    #[facet(figue::subcommand)]
    pub command: GiteaRepoCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum GiteaRepoCommand {
    /// List repositories visible from the tenant.
    List(GiteaRepoListArgs),
    /// Interactively browse repositories visible from the tenant.
    Browse(GiteaRepoBrowseArgs),
    /// Show details for one repository.
    Show(GiteaRepoShowArgs),
}

impl GiteaRepoArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            GiteaRepoCommand::List(args) => Box::pin(args.invoke(executor)).await,
            GiteaRepoCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
            GiteaRepoCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
