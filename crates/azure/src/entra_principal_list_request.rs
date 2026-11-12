use crate::EntraSecurityGroupListRequest;
use crate::EntraServicePrincipalListRequest;
use crate::EntraUserListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
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
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Shared user, security-group, and service-principal inventory for an explicit tenant.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraPrincipalListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl CacheInvalidationKeys for EntraPrincipalListRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![
            self.cache_key()?,
            EntraUserListRequest {
                authentication: self.authentication,
            }
            .cache_key()?,
            EntraSecurityGroupListRequest {
                authentication: self.authentication,
            }
            .cache_key()?,
            EntraServicePrincipalListRequest {
                authentication: self.authentication,
            }
            .cache_key()?,
        ])
    }
}

impl HasCacheKey for EntraPrincipalListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "principals",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraPrincipalListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<EntraPrincipalCollection>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let selection = self.authentication.into();
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
                        let (users, security_groups, service_principals) = executor
                            .join3(
                                |child| async move {
                                    EntraUserListRequest {
                                        authentication: self.authentication,
                                    }
                                    .execute(&child)
                                    .await
                                },
                                |child| async move {
                                    EntraSecurityGroupListRequest {
                                        authentication: self.authentication,
                                    }
                                    .execute(&child)
                                    .await
                                },
                                |child| async move {
                                    EntraServicePrincipalListRequest {
                                        authentication: self.authentication,
                                    }
                                    .execute(&child)
                                    .await
                                },
                            )
                            .await?;
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

cloud_terrastodon_registry::register_thing!(EntraPrincipalListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraPrincipalListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraPrincipalListRequest => Arc<EntraPrincipalCollection>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraPrincipalListRequest,
    name = "principal-list",
    cached
);
