//! Shared command-line authentication selection and metadata resolution contracts.

mod azure_authentication_cli;
mod azure_devops_authentication_cli;

pub use azure_authentication_cli::AzureAuthenticationArgs;
pub use azure_devops_authentication_cli::AzureDevOpsAuthenticationArgs;
