pub mod gitea_user_browse;
pub mod gitea_user_list;
pub mod gitea_user_show;

use clap::Args;
use clap::Subcommand;
use eyre::Result;
pub use gitea_user_browse::GiteaUserBrowseArgs;
pub use gitea_user_list::GiteaUserListArgs;
pub use gitea_user_show::GiteaUserShowArgs;

#[derive(Args, Debug, Clone)]
pub struct GiteaUserArgs {
    #[command(subcommand)]
    pub command: GiteaUserCommand,
}

#[derive(Subcommand, Debug, Clone)]
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
