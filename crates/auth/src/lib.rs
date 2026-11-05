//! Runtime authentication, invocation-owned credential sessions and sensitive token caches.
//! Serializable source and tenant selections are owned by `cloud_terrastodon_auth_types`.

mod azure_cli_local_accounts;
pub use azure_cli_local_accounts::read_azure_cli_local_accounts;
mod auth_bearer_ext;
pub use cloud_terrastodon_command::CancellationScope;
mod auth_context;
mod auth_logout_error;
mod azure_access_token;
mod azure_bearer_token;
mod azure_claims;
mod azure_cli_account_identity;
mod azure_cli_session;
mod azure_cli_session_store;
mod azure_cli_sessions;
mod azure_devops_auth_context;
mod azure_devops_pat;
mod azure_devops_rest_client;
mod azure_tenant_auth_context;
mod azure_token_cache;
mod browser_access_token;
mod browser_opener;
mod browser_session;
#[cfg(windows)]
mod browser_session_metadata;
mod browser_token_cache;
mod browser_token_cache_key;
mod browser_token_response;
mod browser_token_session;
mod credential_authentication_provider;
mod jwt;
mod logout_all_auth_sources;
mod microsoft_graph_access_token;
mod pim_client_id;
mod pim_config;
mod pim_device_code_response;
mod pim_graph_access_token;
mod pim_token_response;
mod resolved_auth_context;
#[cfg(windows)]
mod windows_credential_manager;
#[cfg(windows)]
mod windows_credential_manager_target_name;
mod workload_identity;
mod workload_identity_config;
mod workload_identity_token_response;

pub use auth_bearer_ext::*;
pub use auth_context::*;
pub use azure_access_token::*;
pub use azure_bearer_token::*;
pub use azure_claims::*;
pub(crate) use azure_cli_sessions::AzureCliSessions;
pub use azure_devops_auth_context::*;
pub use azure_devops_pat::*;
pub use azure_devops_rest_client::*;
pub use azure_tenant_auth_context::*;
#[expect(unused_imports)]
pub use azure_token_cache::*;
pub use browser_access_token::login_browser_session;
pub use browser_session::BrowserSession;
pub use browser_token_cache::BrowserTokenCache;
pub use credential_authentication_provider::CredentialAuthenticationProvider;
pub use jwt::*;
pub use logout_all_auth_sources::logout_all_auth_sources;
pub use microsoft_graph_access_token::MicrosoftGraphAccessToken;
pub use pim_client_id::*;
pub use pim_config::*;
pub use pim_graph_access_token::*;
pub use resolved_auth_context::ResolvedAuthContext;
#[cfg(windows)]
pub use windows_credential_manager::*;
#[cfg(windows)]
pub use windows_credential_manager_target_name::*;
pub use workload_identity::*;
pub use workload_identity_config::WorkloadIdentityConfig;

mod container_registry_bearer_challenge;
pub use container_registry_bearer_challenge::ContainerRegistryBearerChallenge;
mod container_registry_access_token_response;
mod container_registry_refresh_token_response;
