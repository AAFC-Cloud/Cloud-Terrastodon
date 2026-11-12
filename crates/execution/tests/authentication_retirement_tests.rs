use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_auth_types::ContainerRegistryBearerChallenge;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_execution::AuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use std::sync::Arc;
use std::time::Duration;
use teamy_cancellation::CancellationToken;
use tokio::sync::Notify;

// This fixture exercises the provider's lease protocol itself, rather than
// substituting fixed credentials as TestAuthenticationProvider does. Retirement
// signals appear only on the newly admitted provider, not its unleased parent.
#[derive(Clone)]
struct LeasedCredentialProbe {
    retirement: CancellationToken,
    started: Arc<Notify>,
    leased: bool,
}

impl AuthenticationProvider for LeasedCredentialProbe {
    fn admit(&self, _selection: &AuthenticationSelection) -> Result<AuthPartition> {
        AuthPartition::new("synthetic-credential-retirement")
    }

    fn lease(
        &self,
        _selection: &AuthenticationSelection,
    ) -> Result<Option<Arc<dyn AuthenticationProvider>>> {
        Ok(Some(Arc::new(Self {
            leased: true,
            ..self.clone()
        })))
    }

    fn retirement_tokens(&self) -> Vec<CancellationToken> {
        if self.leased {
            vec![self.retirement.clone()]
        } else {
            Vec::new()
        }
    }

    fn authorization<'a>(
        &'a self,
        _selection: &'a AuthenticationSelection,
        _resource: AzureCredentialResource,
        executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(async move {
            assert!(
                self.leased,
                "direct authorization must acquire a lease first"
            );
            assert!(
                executor.cache().is_err(),
                "credential infrastructure must remain outside the work cache"
            );
            self.started.notify_one();
            std::future::pending().await
        })
    }

    fn container_registry_authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        _challenge: &'a ContainerRegistryBearerChallenge,
        executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        self.authorization(
            selection,
            AzureCredentialResource::AzureContainerRegistry,
            executor,
        )
    }
}

#[tokio::test]
async fn direct_authorization_observes_retirement_from_its_newly_admitted_lease() -> Result<()> {
    for registry in [false, true] {
        CancellationToken::new().with_async_scope(async |scope| {
            let provider = Arc::new(LeasedCredentialProbe {
                retirement: CancellationToken::new(),
                started: Arc::new(Notify::new()),
                leased: false,
            });
            let executor = CloudTerrastodonExecutor::new(WorkCache::temporary()?, provider.clone(), scope.clone());
            let selection = AuthenticationSelection::AzureBearer(AzureBearerAuthenticationSelection {
                account_id: "11111111-1111-1111-1111-111111111111".parse()?,
                auth_source: AzureBearerAuthSource::Browser,
                tenant_id: "22222222-2222-2222-2222-222222222222".parse()?,
            });
            let challenge = ContainerRegistryBearerChallenge::parse(
                &"https://synthetic.azurecr.io/v2/".parse()?,
                r#"Bearer realm="https://synthetic.azurecr.io/oauth2/token",service="synthetic.azurecr.io",scope="repository:synthetic:pull""#,
            )?;
            let credential = if registry {
                executor.container_registry_authorization(&selection, &challenge)
            } else {
                executor.authorization(&selection, AzureCredentialResource::MicrosoftGraph)
            };
            let retire = async {
                provider.started.notified().await;
                provider.retirement.request_cancel("synthetic account retirement");
            };
                let (credential, ()) = tokio::time::timeout(Duration::from_secs(5), async {
                    tokio::join!(credential, retire)
                }).await?;
            assert!(credential.is_err());
            assert!(!scope.is_cancelled(), "retirement must not cancel the caller");
            eyre::Ok(())
        }).await?;
    }
    Ok(())
}
