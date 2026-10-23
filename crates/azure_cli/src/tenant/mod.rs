pub mod alias;
pub mod azure_tenant_add_cli;
pub mod azure_tenant_discover_cli;
pub mod azure_tenant_forget_cli;
pub mod azure_tenant_list_cli;
pub mod azure_tenant_login_cli;
pub mod azure_tenant_set_auth_source_cli;
pub mod azure_tenant_show_cli;

pub use alias::AzureTenantAliasArgs;
pub use azure_tenant_add_cli::AzureTenantAddArgs;
pub use azure_tenant_discover_cli::AzureTenantDiscoverArgs;
pub use azure_tenant_forget_cli::AzureTenantForgetArgs;
pub use azure_tenant_list_cli::AzureTenantListArgs;
pub use azure_tenant_login_cli::AzureTenantLoginArgs;
pub use azure_tenant_set_auth_source_cli::AzureTenantSetAuthSourceArgs;
pub use azure_tenant_show_cli::AzureTenantShowArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Tenant-related commands for tracked tenant configuration.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureTenantArgs {
    #[facet(figue::subcommand)]
    pub command: AzureTenantCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
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
    /// Log in to an Azure tenant through browser PKCE or an explicitly selected compatibility source.
    Login(AzureTenantLoginArgs),
    /// Set the default authentication source for a tracked Azure tenant.
    SetAuthSource(AzureTenantSetAuthSourceArgs),
}

impl AzureTenantArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self.command {
            AzureTenantCommand::Login(args) => args.invoke(auth_context, cancellation).await?,
            AzureTenantCommand::List(args) => args.invoke(cancellation).await?,
            AzureTenantCommand::Discover(args) => args.invoke(cancellation).await?,
            AzureTenantCommand::Alias(args) => args.invoke(cancellation).await?,
            AzureTenantCommand::Add(args) => args.invoke(cancellation).await?,
            AzureTenantCommand::Show(args) => args.invoke(auth_context, cancellation).await?,
            AzureTenantCommand::Forget(args) => args.invoke(cancellation).await?,
            AzureTenantCommand::SetAuthSource(args) => args.invoke(cancellation).await?,
        }

        Ok(())
    }
}
