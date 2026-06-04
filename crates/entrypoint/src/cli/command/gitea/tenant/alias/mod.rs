pub mod gitea_tenant_alias_add;
pub mod gitea_tenant_alias_list;
pub mod gitea_tenant_alias_remove;

use clap::Args;
use clap::Subcommand;
use eyre::Result;
pub use gitea_tenant_alias_add::GiteaTenantAliasAddArgs;
pub use gitea_tenant_alias_list::GiteaTenantAliasListArgs;
pub use gitea_tenant_alias_remove::GiteaTenantAliasRemoveArgs;

#[derive(Args, Debug, Clone)]
pub struct GiteaTenantAliasArgs {
    #[command(subcommand)]
    pub command: GiteaTenantAliasCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GiteaTenantAliasCommand {
    /// Add one or more aliases to a tracked tenant.
    Add(GiteaTenantAliasAddArgs),
    /// List tracked tenant aliases.
    List(GiteaTenantAliasListArgs),
    /// Remove one or more aliases from a tracked tenant.
    Remove(GiteaTenantAliasRemoveArgs),
}

impl GiteaTenantAliasArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            GiteaTenantAliasCommand::Add(args) => args.invoke().await?,
            GiteaTenantAliasCommand::List(args) => args.invoke().await?,
            GiteaTenantAliasCommand::Remove(args) => args.invoke().await?,
        }
        Ok(())
    }
}
