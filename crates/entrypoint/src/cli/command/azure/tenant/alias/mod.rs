pub mod azure_tenant_alias_add_cli;
pub mod azure_tenant_alias_list_cli;
pub mod azure_tenant_alias_remove_cli;

pub use azure_tenant_alias_add_cli::AzureTenantAliasAddArgs;
pub use azure_tenant_alias_list_cli::AzureTenantAliasListArgs;
pub use azure_tenant_alias_remove_cli::AzureTenantAliasRemoveArgs;
use eyre::Result;

/// Alias-related commands for tracked Azure tenants.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureTenantAliasArgs {
    #[facet(figue::subcommand)]
    pub command: AzureTenantAliasCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureTenantAliasCommand {
    /// Add one or more aliases to a tracked tenant.
    Add(AzureTenantAliasAddArgs),
    /// List tracked tenant aliases.
    List(AzureTenantAliasListArgs),
    /// Remove one or more aliases from a tracked tenant.
    Remove(AzureTenantAliasRemoveArgs),
}

impl AzureTenantAliasArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureTenantAliasCommand::Add(args) => args.invoke().await?,
            AzureTenantAliasCommand::List(args) => args.invoke().await?,
            AzureTenantAliasCommand::Remove(args) => args.invoke().await?,
        }

        Ok(())
    }
}
