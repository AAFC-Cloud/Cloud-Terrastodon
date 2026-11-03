use crate::AzureBearerToken;
use crate::azure_cli_session::AzureCliSession;
use crate::azure_cli_session_store::AzureCliSessionStore;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use eyre::ensure;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Invocation-owned bindings to persistent, tenant-specific Azure CLI logins.
/// Clones share the same bindings; later explicit logins cannot replace them.
#[derive(Clone, Default)]
pub(crate) struct AzureCliSessions {
    sessions: Arc<Mutex<HashMap<AzureTenantId, Arc<AzureCliSession>>>>,
}

impl AzureCliSessions {
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
            AzureCliSessionStore::configured()
                .load(tenant, cancellation)
                .await?,
        );
        sessions.insert(tenant, session.clone());
        Ok(session)
    }

    /// Explicitly create and select a fresh login. Acquisition never calls this.
    pub(crate) async fn login(
        &self,
        tenant: AzureTenantId,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let mut sessions = self.sessions.lock().await;
        ensure!(
            !sessions.contains_key(&tenant),
            "this authentication context is already bound to an Azure CLI session; create a new context to log in again"
        );
        let session = AzureCliSessionStore::configured()
            .login(tenant, cancellation)
            .await?;
        sessions.insert(tenant, Arc::new(session));
        Ok(())
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

    pub(crate) async fn logout_all(cancellation: &CancellationScope) -> Result<()> {
        AzureCliSessionStore::configured()
            .logout_all(cancellation)
            .await
    }
}

impl std::fmt::Debug for AzureCliSessions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("AzureCliSessions([REDACTED])")
    }
}
