pub mod alias;
pub mod gitea_tenant_add;
pub mod gitea_tenant_discover;
pub mod gitea_tenant_forget;
pub mod gitea_tenant_list;
pub mod gitea_tenant_show;

pub use alias::GiteaTenantAliasArgs;
use eyre::Result;
pub use gitea_tenant_add::GiteaTenantAddArgs;
pub use gitea_tenant_discover::GiteaTenantDiscoverArgs;
pub use gitea_tenant_forget::GiteaTenantForgetArgs;
pub use gitea_tenant_list::GiteaTenantListArgs;
pub use gitea_tenant_show::GiteaTenantShowArgs;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaTenantArgs {
    #[facet(figue::subcommand)]
    pub command: GiteaTenantCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum GiteaTenantCommand {
    /// List tracked Gitea tenants.
    List(GiteaTenantListArgs),
    /// Discover Gitea tenants from configured `tea` logins and add them.
    Discover(GiteaTenantDiscoverArgs),
    /// Manage aliases for tracked Gitea tenants.
    Alias(GiteaTenantAliasArgs),
    /// Add a Gitea tenant to the tracked tenant list.
    Add(GiteaTenantAddArgs),
    /// Show details for a tracked Gitea tenant.
    Show(GiteaTenantShowArgs),
    /// Forget a tracked Gitea tenant.
    Forget(GiteaTenantForgetArgs),
}

impl GiteaTenantArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            GiteaTenantCommand::List(args) => args.invoke().await?,
            GiteaTenantCommand::Discover(args) => args.invoke().await?,
            GiteaTenantCommand::Alias(args) => args.invoke().await?,
            GiteaTenantCommand::Add(args) => args.invoke().await?,
            GiteaTenantCommand::Show(args) => args.invoke().await?,
            GiteaTenantCommand::Forget(args) => args.invoke().await?,
        }
        Ok(())
    }
}
