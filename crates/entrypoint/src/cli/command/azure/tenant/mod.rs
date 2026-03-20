pub mod alias;
pub mod azure_tenant_add;
pub mod azure_tenant_discover;
pub mod azure_tenant_forget;
pub mod azure_tenant_list;
pub mod azure_tenant_show;

pub use alias::AzureTenantAliasArgs;
pub use azure_tenant_add::AzureTenantAddArgs;
pub use azure_tenant_discover::AzureTenantDiscoverArgs;
pub use azure_tenant_forget::AzureTenantForgetArgs;
pub use azure_tenant_list::AzureTenantListArgs;
pub use azure_tenant_show::AzureTenantShowArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Tenant-related commands for tracked tenant configuration.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantArgs {
    #[command(subcommand)]
    pub command: AzureTenantCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureTenantCommand {
    /// List tracked Azure tenants.
    List(AzureTenantListArgs),
    /// Discover unique tenant ids from Azure CLI accounts and add them.
    Discover(AzureTenantDiscoverArgs),
    /// Manage aliases for tracked Azure tenants.
    Alias(AzureTenantAliasArgs),
    /// Add a tenant to the tracked tenant list.
    Add(AzureTenantAddArgs),
    /// Show details for a tracked tenant.
    Show(AzureTenantShowArgs),
    /// Forget a tracked tenant.
    Forget(AzureTenantForgetArgs),
}

impl AzureTenantArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureTenantCommand::List(args) => args.invoke().await?,
            AzureTenantCommand::Discover(args) => args.invoke().await?,
            AzureTenantCommand::Alias(args) => args.invoke().await?,
            AzureTenantCommand::Add(args) => args.invoke().await?,
            AzureTenantCommand::Show(args) => args.invoke().await?,
            AzureTenantCommand::Forget(args) => args.invoke().await?,
        }

        Ok(())
    }
}
