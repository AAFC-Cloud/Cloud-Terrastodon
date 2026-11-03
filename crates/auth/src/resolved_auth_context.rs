use crate::AzureBearerToken;
use crate::AzureCliSessions;
use crate::AzureDevOpsPersonalAccessToken;
use crate::BrowserSession;
use crate::BrowserTokenCache;
use crate::WorkloadIdentityConfig;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::OnceCell;
use uuid::Uuid;

pub(crate) type WorkloadIdentityTokenCache =
    Mutex<HashMap<(AzureTenantId, AzureCredentialResource), AzureAccessToken<AzureBearerToken>>>;

/// Credential state belonging to one resolved invocation.
///
/// Fields stay internal because changing a source or principal while retaining
/// its caches or partition would allow one identity to reuse another's results.
#[derive(Debug)]
pub struct ResolvedAuthContext {
    pub(crate) requested_source: AuthSource,
    pub(crate) source: AuthSource,
    pub(crate) workload_identity: Option<WorkloadIdentityConfig>,
    pub(crate) headless: bool,
    pub(crate) browser_token_cache: BrowserTokenCache,
    pub(crate) workload_identity_token_cache: WorkloadIdentityTokenCache,
    pub(crate) browser_session: Option<BrowserSession>,
    pub(crate) session_id: Uuid,
    pub(crate) azure_cli_sessions: AzureCliSessions,
    pub(crate) personal_access_token: OnceCell<Arc<AzureDevOpsPersonalAccessToken>>,
}

impl ResolvedAuthContext {
    pub(crate) fn new(
        requested_source: AuthSource,
        source: AuthSource,
        workload_identity: Option<WorkloadIdentityConfig>,
        headless: bool,
        browser_session: Option<BrowserSession>,
    ) -> Self {
        Self {
            requested_source,
            source,
            workload_identity,
            headless,
            browser_token_cache: BrowserTokenCache::from_session(browser_session.as_ref()),
            workload_identity_token_cache: Mutex::new(HashMap::new()),
            browser_session,
            session_id: Uuid::new_v4(),
            azure_cli_sessions: AzureCliSessions::default(),
            personal_access_token: OnceCell::new(),
        }
    }
}
