use cloud_terrastodon_app::CliOutput;
use facet::Facet;
pub mod alias;
pub mod gitea_tenant_add_cli;
pub mod gitea_tenant_discover_cli;
pub mod gitea_tenant_forget_cli;
pub mod gitea_tenant_list_cli;
pub mod gitea_tenant_show_cli;

pub use alias::GiteaTenantAliasArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
pub use gitea_tenant_add_cli::GiteaTenantAddArgs;
pub use gitea_tenant_discover_cli::GiteaTenantDiscoverArgs;
pub use gitea_tenant_forget_cli::GiteaTenantForgetArgs;
pub use gitea_tenant_list_cli::GiteaTenantListArgs;
pub use gitea_tenant_show_cli::GiteaTenantShowArgs;

#[derive(Facet, Debug, Clone)]
pub struct GiteaTenantArgs {
    #[facet(figue::subcommand)]
    pub command: GiteaTenantCommand,
}

#[derive(Facet, Debug, Clone)]
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            GiteaTenantCommand::List(args) => Box::pin(args.invoke(executor.cancellation())).await,
            GiteaTenantCommand::Discover(args) => Box::pin(args.invoke(executor)).await,
            GiteaTenantCommand::Alias(args) => Box::pin(args.invoke(executor)).await,
            GiteaTenantCommand::Add(args) => Box::pin(args.invoke(executor.cancellation())).await,
            GiteaTenantCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            GiteaTenantCommand::Forget(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
