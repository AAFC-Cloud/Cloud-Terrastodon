use cloud_terrastodon_app::CliOutput;
use facet::Facet;
pub mod gitea_user_browse_cli;
pub mod gitea_user_list_cli;
pub mod gitea_user_show_cli;

use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
pub use gitea_user_browse_cli::GiteaUserBrowseArgs;
pub use gitea_user_list_cli::GiteaUserListArgs;
pub use gitea_user_show_cli::GiteaUserShowArgs;

#[derive(Facet, Debug, Clone)]
pub struct GiteaUserArgs {
    #[facet(figue::subcommand)]
    pub command: GiteaUserCommand,
}

#[derive(Facet, Debug, Clone)]
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            GiteaUserCommand::List(args) => Box::pin(args.invoke(executor)).await,
            GiteaUserCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
            GiteaUserCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
