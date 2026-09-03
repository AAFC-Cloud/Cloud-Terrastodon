pub mod gitea_user_browse_cli;
pub mod gitea_user_list_cli;
pub mod gitea_user_show_cli;

use eyre::Result;
pub use gitea_user_browse_cli::GiteaUserBrowseArgs;
pub use gitea_user_list_cli::GiteaUserListArgs;
pub use gitea_user_show_cli::GiteaUserShowArgs;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaUserArgs {
    #[facet(figue::subcommand)]
    pub command: GiteaUserCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum GiteaUserCommand {
    /// List users visible from the tenant.
    List(GiteaUserListArgs),
    /// Interactively browse users visible from the tenant.
    Browse(GiteaUserBrowseArgs),
    /// Show details for one user.
    Show(GiteaUserShowArgs),
}

impl GiteaUserArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            GiteaUserCommand::List(args) => args.invoke().await?,
            GiteaUserCommand::Browse(args) => args.invoke().await?,
            GiteaUserCommand::Show(args) => args.invoke().await?,
        }
        Ok(())
    }
}
