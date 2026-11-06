//! Serializable authentication selections and sensitive authorization values.
//!
//! These contracts never discover configuration, acquire credentials, or execute
//! requests. Runtime implementations belong to `cloud_terrastodon_auth`.

mod auth_source;
mod authentication_selection;
mod authorization;
mod azure_bearer_auth_source;
mod azure_credential_resource;
mod azure_devops_auth_selection;
mod bearer_authorization;
mod personal_access_token_authorization;

pub use auth_source::AuthSource;
pub use authentication_selection::AuthenticationSelection;
pub use authorization::Authorization;
pub use azure_bearer_auth_source::AzureBearerAuthSource;
pub use azure_credential_resource::AZURE_DEVOPS_DELEGATED_SCOPE;
pub use azure_credential_resource::AZURE_DEVOPS_RESOURCE_ID;
pub use azure_credential_resource::AzureCredentialResource;
pub use azure_devops_auth_selection::AzureDevOpsAuthSelection;
pub use bearer_authorization::BearerAuthorization;
pub use personal_access_token_authorization::PersonalAccessTokenAuthorization;

mod container_registry_bearer_challenge;
pub use container_registry_bearer_challenge::ContainerRegistryBearerChallenge;
