use cloud_terrastodon_app::CliOutput;
use facet::Facet;
pub mod gitea_org_browse_cli;
pub mod gitea_org_list_cli;
pub mod gitea_org_show_cli;

use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
pub use gitea_org_browse_cli::GiteaOrgBrowseArgs;
pub use gitea_org_list_cli::GiteaOrgListArgs;
pub use gitea_org_show_cli::GiteaOrgShowArgs;

#[derive(Facet, Debug, Clone)]
pub struct GiteaOrgArgs {
    #[facet(figue::subcommand)]
    pub command: GiteaOrgCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum GiteaOrgCommand {
    /// List organizations visible from the tenant.
    List(GiteaOrgListArgs),
    /// Interactively browse organizations visible from the tenant.
    Browse(GiteaOrgBrowseArgs),
    /// Show details for one organization.
    Show(GiteaOrgShowArgs),
}

impl GiteaOrgArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            GiteaOrgCommand::List(args) => Box::pin(args.invoke(executor)).await,
            GiteaOrgCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
            GiteaOrgCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
