use crate::EntraGroupGetBatchRequest;
use crate::EntraGroupIdListByMemberRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::sync::Arc;
use tracing::debug;

/// Resolve a principal's transitive group membership to complete group records.
/// [Microsoft Graph v1.0 getMemberGroups](https://learn.microsoft.com/graph/api/directoryobject-getmembergroups?view=graph-rest-1.0).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(traits(Clone))]
pub struct EntraGroupListByMemberRequest {
    pub principal_id: EntraPrincipalId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl HasCacheKey for EntraGroupListByMemberRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        EntraGroupIdListByMemberRequest {
            principal_id: self.principal_id,
            authentication: self.authentication,
        }
        .cache_key()
    }
}

impl CacheInvalidatableRequest for EntraGroupListByMemberRequest {
    fn invalidate<'a>(&'a self, executor: &'a CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()> {
        Box::pin(async move {
            // Group identities are discovered dynamically. Refresh that discovery
            // first, then invalidate exactly the batch that execution will consume.
            let group_ids = EntraGroupIdListByMemberRequest {
                principal_id: self.principal_id,
                authentication: self.authentication,
            }
            .execute(executor)
            .with_invalidation(true)
            .await?;
            EntraGroupGetBatchRequest {
                group_ids: group_ids.value.iter().copied().collect(),
                authentication: self.authentication,
            }
            .invalidate(executor)
            .await
        })
    }
}

impl ExecutableRequest for EntraGroupListByMemberRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraGroup>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                tenant_id = %self.authentication.tenant_id,
                principal_id = %self.principal_id,
                "Fetching Entra groups for principal"
            );
            let group_ids = EntraGroupIdListByMemberRequest {
                principal_id: self.principal_id,
                authentication: self.authentication,
            }
            .execute(&executor)
            .await?;
            let groups = EntraGroupGetBatchRequest {
                group_ids: group_ids.value.iter().copied().collect(),
                authentication: self.authentication,
            }
            .execute(&executor)
            .await?;
            debug!(
                tenant_id = %self.authentication.tenant_id,
                principal_id = %self.principal_id,
                count = groups.len(),
                "Found Entra groups for principal"
            );
            Ok(groups)
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupListByMemberRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupListByMemberRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraGroupListByMemberRequest => Vec<Arc<EntraGroup>>,
    effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraGroupListByMemberRequest,
    name = "entra-group-list-by-member",
    cached
);
