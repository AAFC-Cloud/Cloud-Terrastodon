use crate::AuthenticationProvider;
use crate::CloudTerrastodonExecutor;
use crate::ExecutionFuture;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_auth_types::ContainerRegistryBearerChallenge;
use cloud_terrastodon_cache::AuthPartition;
use eyre::Result;
use eyre::ensure;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// Instrumented authentication session for offline execution tests.
///
/// Available through `test-support`. [`Self::offline`] admits synthetic cache work
/// but rejects credential acquisition. [`Self::new`] supplies
/// a fixed authorization for local transport tests. No constructor resolves credentials.
///
/// Counts record method invocation, even when the returned future is never polled.
/// Source and tenant selections have distinct partitions. Separate fixtures with the
/// same explicit session and generation represent the same identity, permitting tests
/// of differently admitted waiters sharing work. Change `partition_generation` to model
/// a session change. Anonymous work retains its own partition across those changes.
#[derive(Debug)]
pub struct TestAuthenticationProvider {
    pub session: AuthPartition,
    pub allow_anonymous: AtomicBool,
    pub allow_authenticated: AtomicBool,
    pub partition_generation: AtomicUsize,
    pub admissions: AtomicUsize,
    pub authorizations: AtomicUsize,
    pub registry_authorizations: AtomicUsize,
    /// A supplied synthetic credential, or `None` to reject acquisition.
    pub authorization: Option<Authorization>,
}

impl TestAuthenticationProvider {
    /// Admit synthetic selections while prohibiting credential acquisition.
    pub fn offline(session: impl Into<String>) -> Result<Self> {
        Ok(Self {
            session: AuthPartition::new(session)?,
            allow_anonymous: AtomicBool::new(true),
            allow_authenticated: AtomicBool::new(true),
            partition_generation: AtomicUsize::new(0),
            admissions: AtomicUsize::new(0),
            authorizations: AtomicUsize::new(0),
            registry_authorizations: AtomicUsize::new(0),
            authorization: None,
        })
    }

    /// Admit synthetic selections and return the supplied local-test authorization.
    pub fn new(session: impl Into<String>, authorization: Authorization) -> Result<Self> {
        Ok(Self {
            authorization: Some(authorization),
            ..Self::offline(session)?
        })
    }
}

impl AuthenticationProvider for TestAuthenticationProvider {
    fn admit(&self, selection: &AuthenticationSelection) -> Result<AuthPartition> {
        self.admissions.fetch_add(1, Ordering::SeqCst);
        if matches!(selection, AuthenticationSelection::Anonymous) {
            ensure!(
                self.allow_anonymous.load(Ordering::SeqCst),
                "synthetic policy denies anonymous selection"
            );
            return AuthPartition::new(format!("{}-anonymous", self.session.as_str()));
        }
        ensure!(
            self.allow_authenticated.load(Ordering::SeqCst),
            "synthetic policy denies authenticated selection"
        );
        AuthPartition::new(format!(
            "{}-{}-{selection:?}",
            self.session.as_str(),
            self.partition_generation.load(Ordering::SeqCst)
        ))
    }

    fn authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        _resource: AzureCredentialResource,
        _executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        self.authorizations.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            self.admit(selection)?;
            self.authorization
                .clone()
                .ok_or_else(|| eyre::eyre!("offline test forbids credential acquisition"))
        })
    }

    fn container_registry_authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        _challenge: &'a ContainerRegistryBearerChallenge,
        _executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        self.registry_authorizations.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            self.admit(selection)?;
            eyre::bail!("offline test forbids container registry credential exchange");
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_cache::WorkCache;
    use std::sync::Arc;
    use teamy_cancellation::CancellationToken;

    #[test]
    fn independent_sessions_and_revoked_selections_do_not_share_admission() -> Result<()> {
        let first = TestAuthenticationProvider::offline("first-session")?;
        let second = TestAuthenticationProvider::offline("second-session")?;
        let selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken;
        let admitted = first.admit(&selection)?;
        assert_ne!(admitted, second.admit(&selection)?);

        first.allow_authenticated.store(false, Ordering::SeqCst);
        assert!(first.admit(&selection).is_err());
        assert!(first.admit(&AuthenticationSelection::Anonymous).is_ok());

        first.partition_generation.store(1, Ordering::SeqCst);
        first.allow_authenticated.store(true, Ordering::SeqCst);
        assert_ne!(admitted, first.admit(&selection)?);
        Ok(())
    }

    #[tokio::test]
    async fn an_unpolled_authorization_attempt_is_counted_and_offline_acquisition_fails()
    -> Result<()> {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let provider = Arc::new(TestAuthenticationProvider::offline("offline-session")?);
                let executor =
                    CloudTerrastodonExecutor::new(WorkCache::temporary()?, provider.clone(), scope);
                let selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken;
                let attempt = provider.authorization(
                    &selection,
                    AzureCredentialResource::AzureDevOps,
                    &executor,
                );
                assert_eq!(provider.authorizations.load(Ordering::SeqCst), 1);
                drop(attempt);
                assert!(
                    provider
                        .authorization(&selection, AzureCredentialResource::AzureDevOps, &executor,)
                        .await
                        .is_err()
                );
                assert_eq!(provider.authorizations.load(Ordering::SeqCst), 2);
                Ok(())
            })
            .await
    }
}
