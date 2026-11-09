use crate::EntraUserListRequest;
use crate::SecurityGroupListRequest;
use crate::ServicePrincipalListRequest;
use crate::fetch_all_entra_users;
use crate::fetch_all_security_groups;
use crate::fetch_all_service_principals;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraPrincipal;
use cloud_terrastodon_azure_types::EntraPrincipalCollection;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::try_join;
use tracing::debug;

/// Shared user, security-group, and service-principal inventory for an explicit tenant.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct PrincipalListRequest {
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
}

#[track_caller]
pub fn fetch_all_principals<'a>(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, PrincipalListRequest> {
    executor.execute(PrincipalListRequest {
        tenant_id,
        auth_source,
    })
}

impl CacheInvalidationKeys for PrincipalListRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![
            self.cache_key()?,
            EntraUserListRequest {
                tenant_id: self.tenant_id,
                auth_source: self.auth_source,
            }
            .cache_key()?,
            SecurityGroupListRequest {
                tenant_id: self.tenant_id,
                auth_source: self.auth_source,
            }
            .cache_key()?,
            ServicePrincipalListRequest {
                tenant_id: self.tenant_id,
                auth_source: self.auth_source,
            }
            .cache_key()?,
        ])
    }
}

impl HasCacheKey for PrincipalListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "principals",
            self.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for PrincipalListRequest {
    type Error = eyre::Report;
    type Output = Arc<EntraPrincipalCollection>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let selection = AuthenticationSelection::AzureBearer {
                tenant_id: self.tenant_id,
                auth_source: self.auth_source,
            };
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    self.cache_key()?,
                    CacheOperationOwner::of::<Self>(),
                    &selection,
                    NonEmptyString::new(facet_json::to_string(&self)?)?,
                    WorkReuse::ReusableRead,
                    move |executor| async move {
                        debug!(
                            "Fetching principals (users, security groups, and service principals)"
                        );
                        let (users, security_groups, service_principals) = try_join!(
                            fetch_all_entra_users(self.tenant_id, self.auth_source, &executor),
                            fetch_all_security_groups(self.tenant_id, self.auth_source, &executor),
                            fetch_all_service_principals(
                                self.tenant_id,
                                self.auth_source,
                                &executor
                            )
                        )?;
                        let principals = EntraPrincipalCollection::new(
                            users
                                .iter()
                                .cloned()
                                .map(EntraPrincipal::from)
                                .chain(security_groups.iter().cloned().map(EntraPrincipal::from))
                                .chain(
                                    service_principals.iter().cloned().map(EntraPrincipal::from),
                                ),
                        );
                        debug!("Found {} principals", principals.len());
                        Ok(WorkOutput::new(principals, ArtifactObservation::now()))
                    },
                )?
                .await
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::fetch_all_principals;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let found = fetch_all_principals(
                    auth_context.tenant_id,
                    auth_context.auth_source()?,
                    &executor,
                )
                .await?;
                assert!(found.len() > 10);
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(PrincipalListRequest);
cloud_terrastodon_registry::register_arbitrary!(PrincipalListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(PrincipalListRequest => Arc<EntraPrincipalCollection>, effects = [Read]);
