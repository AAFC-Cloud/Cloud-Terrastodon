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
pub mod storage;
pub mod subscription;
pub mod tag;
pub mod tenant;
pub mod virtual_network;
pub mod vm;

mod audit_azure;
#[cfg(test)]
mod audit_policy_tests;
mod browse_application_registrations;
mod browse_policy_assignments;
mod browse_service_principals;
mod browse_users;
mod pim_activate;
mod pim_kind;
mod policy_compliance_row;
#[cfg(test)]
mod storage_audit_invalidation_tests;
#[cfg(test)]
mod storage_audit_policy_tests;

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

mod azure_audit_finding;
pub use azure_audit_finding::AzureAuditFinding;

mod azure_audit_password_credential_status;
pub use azure_audit_password_credential_status::AzureAuditPasswordCredentialStatus;

mod azure_audit_report;
pub use azure_audit_report::AzureAuditReport;

mod azure_unknown_principal_audit_request;
pub use azure_unknown_principal_audit_request::AzureUnknownPrincipalAuditRequest;

mod azure_service_principal_password_audit_request;
pub use azure_service_principal_password_audit_request::AzureServicePrincipalPasswordAuditRequest;

mod azure_resource_tag_audit_request;
pub use azure_resource_tag_audit_request::AzureResourceTagAuditRequest;

mod azure_admin_correspondence_audit_request;
pub use azure_admin_correspondence_audit_request::AzureAdminCorrespondenceAuditRequest;

mod azure_storage_account_audit_request;
pub use azure_storage_account_audit_request::AzureStorageAccountAuditRequest;

mod azure_storage_account_audit_report;
pub use azure_storage_account_audit_report::AzureStorageAccountAuditReport;
