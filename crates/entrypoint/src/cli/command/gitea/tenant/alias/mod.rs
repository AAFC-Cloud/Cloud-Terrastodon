pub mod gitea_tenant_alias_add_cli;
pub mod gitea_tenant_alias_list_cli;
pub mod gitea_tenant_alias_remove_cli;

use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
pub use gitea_tenant_alias_add_cli::GiteaTenantAliasAddArgs;
pub use gitea_tenant_alias_list_cli::GiteaTenantAliasListArgs;
pub use gitea_tenant_alias_remove_cli::GiteaTenantAliasRemoveArgs;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaTenantAliasArgs {
    #[facet(figue::subcommand)]
    pub command: GiteaTenantAliasCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum GiteaTenantAliasCommand {
    /// Add one or more aliases to a tracked tenant.
    Add(GiteaTenantAliasAddArgs),
    /// List tracked tenant aliases.
    List(GiteaTenantAliasListArgs),
    /// Remove one or more aliases from a tracked tenant.
    Remove(GiteaTenantAliasRemoveArgs),
}

impl GiteaTenantAliasArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<()> {
        match self.command {
            GiteaTenantAliasCommand::Add(args) => args.invoke(cancellation).await?,
            GiteaTenantAliasCommand::List(args) => args.invoke(cancellation).await?,
            GiteaTenantAliasCommand::Remove(args) => args.invoke(cancellation).await?,
        }
        Ok(())
    }
}
