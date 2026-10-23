//! Azure command parsing, dispatch, and presentation built on the Azure SDK.

pub mod app_service;
pub mod application_gateway;
pub mod audit_cli;
pub mod azure_command_cli;
pub mod cognitive_services;
pub mod container_instance;
pub mod entra;
pub mod find_cli;
pub mod network_interface;
pub mod pim;
pub mod policy;
pub mod private_endpoint;
pub mod public_ip;
pub mod resource;
pub mod resource_group;
pub mod role;
pub mod subscription;
pub mod tag;
pub mod tenant;
pub mod virtual_network;
pub mod vm;

mod audit_azure;
mod browse_application_registrations;
mod browse_policy_assignments;
mod browse_service_principals;
mod browse_users;
mod pim_activate;
mod pim_kind;
mod policy_compliance_row;

pub use audit_azure::audit_azure;
pub use azure_command_cli::AzureArgs;
pub use azure_command_cli::AzureCommand;
pub use browse_application_registrations::browse_application_registrations;
pub use browse_policy_assignments::browse_policy_assignments;
pub use browse_service_principals::browse_service_principals;
pub use browse_users::browse_users;
pub use pim_activate::build_duration_choices;
pub use pim_activate::pim_activate;
pub use pim_activate::pim_activate_azurerm;
pub use pim_activate::pim_activate_entra;
