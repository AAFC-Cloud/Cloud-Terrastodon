//! Shared account registration, authentication selection, and metadata resolution commands.

mod account_add_browser_cli;
mod azure_authentication_cli;
mod azure_devops_authentication_cli;

pub use account_add_browser_cli::AccountAddBrowserArgs;
pub use azure_authentication_cli::AzureAuthenticationArgs;
pub use azure_devops_authentication_cli::AzureDevOpsAuthenticationArgs;
