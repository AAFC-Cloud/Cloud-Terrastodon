pub mod gitea_repo_browse;
pub mod gitea_repo_list;
pub mod gitea_repo_show;

use eyre::Result;
pub use gitea_repo_browse::GiteaRepoBrowseArgs;
pub use gitea_repo_list::GiteaRepoListArgs;
pub use gitea_repo_show::GiteaRepoShowArgs;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaRepoArgs {
    #[facet(figue::subcommand)]
    pub command: GiteaRepoCommand,
}

#[derive(facet::Facet, Debug, Clone)]
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
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            GiteaRepoCommand::List(args) => args.invoke().await?,
            GiteaRepoCommand::Browse(args) => args.invoke().await?,
            GiteaRepoCommand::Show(args) => args.invoke().await?,
        }
        Ok(())
    }
}
