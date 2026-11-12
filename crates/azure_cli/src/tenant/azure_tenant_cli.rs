use super::AzureTenantAddArgs;
use super::AzureTenantAliasArgs;
use super::AzureTenantDefaultArgs;
use super::AzureTenantDiscoverArgs;
use super::AzureTenantForgetArgs;
use super::AzureTenantListArgs;
use super::AzureTenantShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Tenant-related commands for tracked tenant configuration.
#[derive(Facet, Debug, Clone)]
pub struct AzureTenantArgs {
    #[facet(figue::subcommand)]
    pub command: AzureTenantCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureTenantCommand {
    /// List tracked Azure tenants.
    List(AzureTenantListArgs),
    /// Configure the Cloud Terrastodon tenant default.
    Default(AzureTenantDefaultArgs),
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let cancellation = executor.cancellation();
        match self.command {
            AzureTenantCommand::Default(args) => Box::pin(args.invoke(cancellation)).await,
            AzureTenantCommand::List(args) => Box::pin(args.invoke(cancellation)).await,
            AzureTenantCommand::Discover(args) => Box::pin(args.invoke(cancellation)).await,
            AzureTenantCommand::Alias(args) => Box::pin(args.invoke(cancellation)).await,
            AzureTenantCommand::Add(args) => Box::pin(args.invoke(cancellation)).await,
            AzureTenantCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            AzureTenantCommand::Forget(args) => Box::pin(args.invoke(cancellation)).await,
        }
    }
}
