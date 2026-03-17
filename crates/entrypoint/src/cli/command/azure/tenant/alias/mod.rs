pub mod azure_tenant_alias_add;
pub mod azure_tenant_alias_list;
pub mod azure_tenant_alias_remove;

pub use azure_tenant_alias_add::AzureTenantAliasAddArgs;
pub use azure_tenant_alias_list::AzureTenantAliasListArgs;
pub use azure_tenant_alias_remove::AzureTenantAliasRemoveArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Alias-related commands for tracked Azure tenants.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantAliasArgs {
    #[command(subcommand)]
    pub command: AzureTenantAliasCommand,
}

#[derive(Subcommand, Debug, Clone)]
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