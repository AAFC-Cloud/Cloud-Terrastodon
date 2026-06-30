pub mod gitea_org_browse;
pub mod gitea_org_list;
pub mod gitea_org_show;

use eyre::Result;
pub use gitea_org_browse::GiteaOrgBrowseArgs;
pub use gitea_org_list::GiteaOrgListArgs;
pub use gitea_org_show::GiteaOrgShowArgs;

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
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            GiteaOrgCommand::List(args) => args.invoke().await?,
            GiteaOrgCommand::Browse(args) => args.invoke().await?,
            GiteaOrgCommand::Show(args) => args.invoke().await?,
        }
        Ok(())
    }
}
