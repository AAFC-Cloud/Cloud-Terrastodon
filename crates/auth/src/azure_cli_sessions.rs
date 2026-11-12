use crate::AzureBearerToken;
use crate::azure_cli_session::AzureCliSession;
use crate::azure_cli_session_store::AzureCliSessionStore;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Invocation-owned bindings to persistent, tenant-specific Azure CLI logins.
/// Clones share the same bindings; later explicit logins cannot replace them.
#[derive(Clone, Default)]
pub(crate) struct AzureCliSessions {
    sessions: Arc<Mutex<HashMap<AzureTenantId, Arc<AzureCliSession>>>>,
    store: Option<AzureCliSessionStore>,
}

impl AzureCliSessions {
    pub(crate) fn at(root: std::path::PathBuf) -> Self {
        Self {
            sessions: Arc::default(),
            store: Some(AzureCliSessionStore::at(root)),
        }
    }
    async fn session(
        &self,
        tenant: AzureTenantId,
        cancellation: &CancellationScope,
    ) -> Result<Arc<AzureCliSession>> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get(&tenant) {
            return Ok(session.clone());
        }
        let session = Arc::new(
            self.store
                .as_ref()
                .ok_or_else(|| eyre::eyre!("no named-account Azure CLI store was supplied"))?
                .load(tenant, cancellation)
                .await?,
        );
        sessions.insert(tenant, session.clone());
        Ok(session)
    }

    pub(crate) async fn fetch_bearer_access_token(
        &self,
        tenant: AzureTenantId,
        resource: AzureCredentialResource,
        cancellation: &CancellationScope,
    ) -> Result<AzureAccessToken<AzureBearerToken>> {
        self.session(tenant, cancellation)
            .await?
            .fetch_token(resource, cancellation)
            .await
    }
}

impl std::fmt::Debug for AzureCliSessions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("AzureCliSessions([REDACTED])")
    }
}
