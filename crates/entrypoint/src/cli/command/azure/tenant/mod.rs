pub mod alias;
pub mod azure_tenant_add;
pub mod azure_tenant_discover;
pub mod azure_tenant_forget;
pub mod azure_tenant_list;
pub mod azure_tenant_login;
pub mod azure_tenant_set_auth_source;
pub mod azure_tenant_show;

pub use alias::AzureTenantAliasArgs;
pub use azure_tenant_add::AzureTenantAddArgs;
pub use azure_tenant_discover::AzureTenantDiscoverArgs;
pub use azure_tenant_forget::AzureTenantForgetArgs;
pub use azure_tenant_list::AzureTenantListArgs;
pub use azure_tenant_login::AzureTenantLoginArgs;
pub use azure_tenant_set_auth_source::AzureTenantSetAuthSourceArgs;
pub use azure_tenant_show::AzureTenantShowArgs;
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self.command {
            AzureTenantCommand::Login(args) => args.invoke(auth_context).await?,
            AzureTenantCommand::List(args) => args.invoke().await?,
            AzureTenantCommand::Discover(args) => args.invoke().await?,
            AzureTenantCommand::Alias(args) => args.invoke().await?,
            AzureTenantCommand::Add(args) => args.invoke().await?,
            AzureTenantCommand::Show(args) => args.invoke(auth_context).await?,
            AzureTenantCommand::Forget(args) => args.invoke().await?,
            AzureTenantCommand::SetAuthSource(args) => args.invoke().await?,
        }

        Ok(())
    }
}
