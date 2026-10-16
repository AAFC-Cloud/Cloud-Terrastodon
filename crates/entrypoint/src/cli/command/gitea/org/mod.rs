pub mod gitea_org_browse_cli;
pub mod gitea_org_list_cli;
pub mod gitea_org_show_cli;

use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
pub use gitea_org_browse_cli::GiteaOrgBrowseArgs;
pub use gitea_org_list_cli::GiteaOrgListArgs;
pub use gitea_org_show_cli::GiteaOrgShowArgs;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaOrgArgs {
    #[facet(figue::subcommand)]
    pub command: GiteaOrgCommand,
}

#[derive(facet::Facet, Debug, Clone)]
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
    pub async fn invoke(
        self,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self.command {
            GiteaOrgCommand::List(args) => args.invoke(cancellation).await?,
            GiteaOrgCommand::Browse(args) => args.invoke(cancellation).await?,
            GiteaOrgCommand::Show(args) => args.invoke(cancellation).await?,
        }
        Ok(())
    }
}
