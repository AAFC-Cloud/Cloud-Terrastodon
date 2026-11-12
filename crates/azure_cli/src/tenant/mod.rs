pub mod alias;
pub mod azure_tenant_add_cli;
pub mod azure_tenant_cli;
pub mod azure_tenant_discover_cli;
pub mod azure_tenant_forget_cli;
pub mod azure_tenant_list_cli;
pub mod azure_tenant_show_cli;

pub use alias::AzureTenantAliasArgs;
pub use azure_tenant_add_cli::AzureTenantAddArgs;
pub use azure_tenant_cli::AzureTenantArgs;
pub use azure_tenant_cli::AzureTenantCommand;
pub use azure_tenant_discover_cli::AzureTenantDiscoverArgs;
pub use azure_tenant_forget_cli::AzureTenantForgetArgs;
pub use azure_tenant_list_cli::AzureTenantListArgs;
pub use azure_tenant_show_cli::AzureTenantShowArgs;

pub mod azure_tenant_default_cli;
pub use azure_tenant_default_cli::AzureTenantDefaultArgs;
