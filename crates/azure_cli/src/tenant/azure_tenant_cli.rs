use super::AzureTenantAddArgs;
use super::AzureTenantAliasArgs;
use super::AzureTenantDiscoverArgs;
use super::AzureTenantForgetArgs;
use super::AzureTenantListArgs;
use super::AzureTenantLoginArgs;
use super::AzureTenantLogoutArgs;
use super::AzureTenantSetAuthSourceArgs;
use super::AzureTenantShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::AuthContext;
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
    /// Log out of all stored authentication sources, tenants, and organizations.
    Logout(AzureTenantLogoutArgs),
    /// Set the default authentication source for a tracked Azure tenant.
    SetAuthSource(AzureTenantSetAuthSourceArgs),
}

impl AzureTenantArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<CliOutput> {
        let cancellation = executor.cancellation();
        match self.command {
            AzureTenantCommand::Login(args) => {
                Box::pin(args.invoke(auth_context, cancellation)).await?
            }
            AzureTenantCommand::Logout(args) => return Box::pin(args.invoke(cancellation)).await,
            AzureTenantCommand::List(args) => Box::pin(args.invoke(cancellation)).await?,
            AzureTenantCommand::Discover(args) => Box::pin(args.invoke(cancellation)).await?,
            AzureTenantCommand::Alias(args) => Box::pin(args.invoke(cancellation)).await?,
            AzureTenantCommand::Add(args) => Box::pin(args.invoke(cancellation)).await?,
            AzureTenantCommand::Show(args) => Box::pin(args.invoke(auth_context, executor)).await?,
            AzureTenantCommand::Forget(args) => Box::pin(args.invoke(cancellation)).await?,
            AzureTenantCommand::SetAuthSource(args) => Box::pin(args.invoke(cancellation)).await?,
        }

        Ok(CliOutput::none())
    }
}
