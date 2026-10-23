//! Azure DevOps command parsing, dispatch, and presentation built on the SDK.

pub mod agent;
pub mod agent_package;
mod audit_azure_devops;
pub mod audit_cli;
pub mod azure_devops_command_cli;
pub mod azure_devops_rest_command_cli;
pub mod build;
pub mod group;
pub mod license_entitlement;
pub mod project;
pub mod repo;
pub mod service_endpoint;
pub mod team;
pub mod test;
pub mod work_item;

pub use audit_azure_devops::audit_azure_devops;
pub use azure_devops_command_cli::AzureDevOpsArgs;
pub use azure_devops_command_cli::AzureDevOpsCommand;
