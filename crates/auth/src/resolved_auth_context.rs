use crate::AzureBearerToken;
use crate::AzureCliSessions;
use crate::AzureDevOpsPersonalAccessToken;
use crate::BrowserSession;
use crate::BrowserTokenCache;
use crate::WorkloadIdentityConfig;
use crate::cloud_terrastodon_account_admission::CloudTerrastodonAccountAdmission;
use crate::cloud_terrastodon_account_secret_store::CloudTerrastodonAccountSecretStore;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::OnceCell;
use uuid::Uuid;

pub(crate) type WorkloadIdentityTokenCache =
    Mutex<HashMap<(AzureTenantId, AzureCredentialResource), AzureAccessToken<AzureBearerToken>>>;

/// Credential state belonging to one selected account mechanism and session.
///
/// Fields stay internal because changing a source or principal while retaining
/// its caches or partition would allow one identity to reuse another's results.
#[derive(Debug)]
pub struct ResolvedAuthContext {
    pub(crate) admission: Arc<CloudTerrastodonAccountAdmission>,
    pub(crate) account_id: CloudTerrastodonAccountId,
    pub(crate) tenant_id: Option<AzureTenantId>,
    pub(crate) organization: Option<AzureDevOpsOrganizationUrl>,
    pub(crate) credential_store: Option<CloudTerrastodonAccountSecretStore>,
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
        source: AuthSource,
        workload_identity: Option<WorkloadIdentityConfig>,
        headless: bool,
        browser_session: Option<BrowserSession>,
    ) -> Self {
        Self {
            admission: Arc::new(CloudTerrastodonAccountAdmission::new()),
            account_id: CloudTerrastodonAccountId::new(Uuid::new_v4()),
            tenant_id: None,
            organization: None,
            credential_store: None,
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
