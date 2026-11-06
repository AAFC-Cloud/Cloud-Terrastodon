use crate::AuthContext;
use crate::fetch_azure_bearer_access_token;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_auth_types::ContainerRegistryBearerChallenge;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_execution::AuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use eyre::ensure;

/// Adapts an explicitly owned credential session to executor authentication.
///
/// Construction does not resolve configuration or acquire credentials. Clones,
/// including adapters built from clones of the same [`AuthContext`], retain its
/// admitted principal partition and sensitive token caches. Independent contexts
/// preserve the partition policy established by [`AuthContext::admit`].
#[derive(Clone)]
pub struct CredentialAuthenticationProvider {
    // Replacing the context after admission could pair a cached partition with
    // another credential session. A provider therefore owns one immutable context.
    auth_context: AuthContext,
}

impl CredentialAuthenticationProvider {
    /// Bind already-selected authentication without discovering configuration,
    /// loading credentials, or changing the supplied context's session identity.
    pub fn new(auth_context: AuthContext) -> Self {
        Self { auth_context }
    }
}

impl AuthenticationProvider for CredentialAuthenticationProvider {
    fn admit(&self, selection: &AuthenticationSelection) -> Result<AuthPartition> {
        self.auth_context.admit(selection)
    }

    fn authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        resource: AzureCredentialResource,
        executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(async move {
            // Check again at acquisition so direct provider calls cannot bypass
            // the source and tenant policy enforced before executor cache reuse.
            self.admit(selection)?;
            match *selection {
                AuthenticationSelection::Anonymous => Ok(Authorization::Anonymous),
                AuthenticationSelection::AzureBearer { tenant_id, .. } => {
                    let token = fetch_azure_bearer_access_token(
                        &self.auth_context,
                        tenant_id,
                        resource,
                        executor.cancellation(),
                    )
                    .await?;
                    Authorization::bearer(token.access_token.as_str())
                }
                AuthenticationSelection::AzureDevOpsPersonalAccessToken => {
                    ensure!(
                        resource == AzureCredentialResource::AzureDevOps,
                        "Azure DevOps personal access tokens cannot authorize another resource audience"
                    );
                    let token = self
                        .auth_context
                        .personal_access_token(executor.cancellation())
                        .await?;
                    Authorization::personal_access_token(token.as_str())
                }
            }
        })
    }

    fn container_registry_authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        challenge: &'a ContainerRegistryBearerChallenge,
        executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(async move {
            self.admit(selection)?;
            let AuthenticationSelection::AzureBearer { tenant_id, .. } = selection else {
                eyre::bail!(
                    "Container Registry requires an explicitly tenant-bound bearer selection"
                );
            };
            crate::container_registry_authorization::authorize_container_registry(
                challenge,
                &self.auth_context,
                *tenant_id,
                executor.cancellation(),
            )
            .await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CredentialAuthenticationProvider;
    use crate::AuthContext;
    use crate::AzureDevOpsPersonalAccessToken;
    use crate::WorkloadIdentityConfig;
    use cloud_terrastodon_auth_types::AuthSource;
    use cloud_terrastodon_auth_types::AuthenticationSelection;
    use cloud_terrastodon_auth_types::Authorization;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_auth_types::AzureCredentialResource;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_execution::AuthenticationProvider;
    use cloud_terrastodon_execution::CloudTerrastodonExecutor;
    use eyre::Result;
    use std::sync::Arc;

    #[test]
    fn admission_preserves_disabled_and_source_tenant_mismatch_policy() -> Result<()> {
        let tenant_id = "11111111-1111-1111-1111-111111111111".parse()?;
        let bearer = AuthenticationSelection::AzureBearer {
            auth_source: AzureBearerAuthSource::WorkloadIdentity,
            tenant_id,
        };
        for context in [AuthContext::None, AuthContext::Unresolved] {
            let provider = CredentialAuthenticationProvider::new(context);
            assert!(provider.admit(&bearer).is_err());
            assert!(
                provider
                    .admit(&AuthenticationSelection::AzureDevOpsPersonalAccessToken)
                    .is_err()
            );
            assert!(provider.admit(&AuthenticationSelection::Anonymous).is_ok());
        }

        let provider = CredentialAuthenticationProvider::new(
            AuthContext::explicit(AuthSource::WorkloadIdentity).with_workload_identity(
                WorkloadIdentityConfig {
                    tenant_id,
                    client_id: "22222222-2222-2222-2222-222222222222".parse()?,
                    federated_token: "synthetic-unused-assertion".to_owned(),
                },
            )?,
        );
        assert!(provider.admit(&bearer).is_ok());
        assert!(
            provider
                .admit(&AuthenticationSelection::AzureBearer {
                    auth_source: AzureBearerAuthSource::AzureCli,
                    tenant_id,
                })
                .is_err()
        );
        assert!(
            provider
                .admit(&AuthenticationSelection::AzureBearer {
                    auth_source: AzureBearerAuthSource::WorkloadIdentity,
                    tenant_id: "33333333-3333-3333-3333-333333333333".parse()?,
                })
                .is_err()
        );
        Ok(())
    }

    #[test]
    fn rewrapping_an_owned_session_preserves_its_partition() -> Result<()> {
        let context = AuthContext::explicit(AuthSource::PersonalAccessToken);
        let provider = CredentialAuthenticationProvider::new(context.clone());
        let selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken;
        let partition = provider.admit(&selection)?;
        assert_eq!(partition, provider.clone().admit(&selection)?);
        assert_eq!(
            partition,
            CredentialAuthenticationProvider::new(context).admit(&selection)?
        );
        assert_ne!(
            partition,
            CredentialAuthenticationProvider::new(AuthContext::explicit(
                AuthSource::PersonalAccessToken
            ))
            .admit(&selection)?
        );
        Ok(())
    }

    #[tokio::test]
    async fn anonymous_authorization_needs_no_resolved_credentials() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|scope| async move {
                // WorkCache construction only validates this explicit path. The test
                // performs authentication alone and never opens a cache directory.
                let cache = WorkCache::new(if cfg!(windows) {
                    "C:/unused-auth-provider-cache"
                } else {
                    "/unused-auth-provider-cache"
                })?;
                for context in [AuthContext::Unresolved, AuthContext::None] {
                    let provider = Arc::new(CredentialAuthenticationProvider::new(context));
                    let executor = CloudTerrastodonExecutor::new(
                        cache.clone(),
                        provider.clone(),
                        scope.clone(),
                    );
                    let anonymous = executor
                        .authorization(
                            &AuthenticationSelection::Anonymous,
                            AzureCredentialResource::AzureResourceManager,
                        )
                        .await?;
                    assert!(matches!(anonymous, Authorization::Anonymous));
                    assert!(anonymous.header().is_none());

                    // Even calling the provider directly must retain its admission
                    // check instead of attempting a credential fallback.
                    let bearer = AuthenticationSelection::AzureBearer {
                        auth_source: AzureBearerAuthSource::AzureCli,
                        tenant_id: "11111111-1111-1111-1111-111111111111".parse()?,
                    };
                    assert!(
                        provider
                            .authorization(
                                &bearer,
                                AzureCredentialResource::AzureResourceManager,
                                &executor,
                            )
                            .await
                            .is_err()
                    );
                }
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn pat_audience_is_checked_before_loading_credentials() -> Result<()> {
        let context = AuthContext::explicit(AuthSource::PersonalAccessToken);
        let AuthContext::Resolved(state) = &context else {
            panic!("explicit PAT context must be resolved");
        };
        let provider = Arc::new(CredentialAuthenticationProvider::new(context.clone()));
        CancellationToken::new()
            .cancellable_scope(|scope| async move {
                let cache = WorkCache::new(if cfg!(windows) {
                    "C:/unused-auth-provider-cache"
                } else {
                    "/unused-auth-provider-cache"
                })?;
                let executor = CloudTerrastodonExecutor::new(cache, provider.clone(), scope);
                let selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken;
                for audience in [
                    AzureCredentialResource::AzureResourceManager,
                    AzureCredentialResource::MicrosoftGraph,
                ] {
                    assert!(
                        provider
                            .authorization(&selection, audience, &executor)
                            .await
                            .is_err()
                    );
                    assert!(state.personal_access_token.get().is_none());
                }

                // Capture only a synthetic PAT so the accepted path cannot consult
                // a credential manager or environment variable.
                state
                    .personal_access_token
                    .set(Arc::new(AzureDevOpsPersonalAccessToken::new(
                        "synthetic-captured-provider-pat",
                    )))?;
                let authorization = executor
                    .authorization(&selection, AzureCredentialResource::AzureDevOps)
                    .await?;
                assert!(matches!(
                    authorization,
                    Authorization::PersonalAccessToken(_)
                ));
                assert!(authorization.header().unwrap().is_sensitive());
                Ok(())
            })
            .await
    }
}
