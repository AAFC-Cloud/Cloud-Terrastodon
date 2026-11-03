use crate::BrowserSession;
use crate::browser_token_session::BrowserTokenSession;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use eyre::Result;
use eyre::bail;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;

/// Coordinates one browser credential session within an authentication context.
///
/// Tenant and client admission is immutable. The acquisition lock covers the
/// complete cache-check, login/refresh, and publication lifecycle across resources,
/// keeping refresh-token rotation and account selection exclusive. Clones share
/// the same admission and credentials; separately constructed caches do not.
#[derive(Clone, Default)]
pub struct BrowserTokenCache {
    tenant_id: Arc<OnceLock<AzureTenantId>>,
    client_id: Arc<OnceLock<EntraApplicationClientId>>,
    session: Arc<Mutex<BrowserTokenSession>>,
}

impl BrowserTokenCache {
    pub(crate) fn from_session(session: Option<&BrowserSession>) -> Self {
        Self {
            tenant_id: Arc::new(match session {
                Some(session) => OnceLock::from(session.tenant_id),
                None => OnceLock::new(),
            }),
            client_id: Arc::new(match session {
                Some(session) => OnceLock::from(session.client_id),
                None => OnceLock::new(),
            }),
            session: Arc::new(Mutex::new(BrowserTokenSession::from_session(session))),
        }
    }

    /// Bind this session to its first tenant, or check its stored tenant binding.
    pub(crate) fn admit_tenant(&self, tenant_id: AzureTenantId) -> Result<()> {
        if self.tenant_id.get_or_init(|| tenant_id) != &tenant_id {
            bail!("browser authentication session is bound to a different tenant");
        }
        Ok(())
    }

    pub(crate) async fn acquire(
        &self,
        tenant_id: AzureTenantId,
        client_id: EntraApplicationClientId,
    ) -> Result<MutexGuard<'_, BrowserTokenSession>> {
        self.admit_tenant(tenant_id)?;
        if self.client_id.get_or_init(|| client_id) != &client_id {
            bail!("browser authentication session is bound to a different client registration");
        }
        Ok(self.session.lock().await)
    }
}

impl std::fmt::Debug for BrowserTokenCache {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BrowserTokenCache")
            .field("session", &"***redacted***")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn clones_preserve_tenant_and_client_admission_before_acquisition() {
        let cache = BrowserTokenCache::default();
        let tenant = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let other_tenant = "33333333-3333-3333-3333-333333333333".parse().unwrap();
        let client = "22222222-2222-2222-2222-222222222222".parse().unwrap();
        let other_client = "44444444-4444-4444-4444-444444444444".parse().unwrap();
        let mut session = cache.acquire(tenant, client).await.unwrap();
        session.begin_initial_login(true).unwrap();
        let clone = cache.clone();
        assert!(clone.admit_tenant(other_tenant).is_err());
        // Mismatched admission must fail even while another acquisition holds the lock.
        assert!(clone.acquire(tenant, other_client).await.is_err());
        drop(session);
        assert!(
            clone
                .acquire(tenant, client)
                .await
                .unwrap()
                .begin_initial_login(true)
                .is_err()
        );
    }

    #[tokio::test]
    async fn saved_session_is_an_immutable_credential_snapshot() {
        let mut saved = BrowserSession {
            tenant_id: "11111111-1111-1111-1111-111111111111".parse().unwrap(),
            client_id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
            refresh_token: "synthetic-original".to_owned(),
        };
        let cache = BrowserTokenCache::from_session(Some(&saved));
        saved.refresh_token = "synthetic-replacement".to_owned();
        let mut session = cache
            .acquire(saved.tenant_id, saved.client_id)
            .await
            .unwrap();
        assert_eq!(session.refresh_token(), Some("synthetic-original"));
        assert!(session.begin_initial_login(true).is_err());
    }
}
