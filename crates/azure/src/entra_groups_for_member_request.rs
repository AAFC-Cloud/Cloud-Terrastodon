use crate::EntraGroupByIdRequest;
use crate::EntraMemberGroupIdsRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraGroupId;
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
pub struct EntraGroupsForMemberRequest {
    pub principal_id: EntraPrincipalId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl EntraGroupsForMemberRequest {
    fn group_lookup_request(&self, group_ids: &[EntraGroupId]) -> EntraGroupByIdRequest {
        // Membership is a set. The lookup and refresh must use the same batch
        // identity regardless of the order in which Graph returns its IDs.
        let mut group_ids = group_ids.to_vec();
        group_ids.sort_unstable_by_key(|id| id.0);
        group_ids.dedup();
        EntraGroupByIdRequest {
            group_ids,
            authentication: self.authentication,
        }
    }
}

impl HasCacheKey for EntraGroupsForMemberRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        EntraMemberGroupIdsRequest {
            principal_id: self.principal_id,
            authentication: self.authentication,
        }
        .cache_key()
    }
}

impl CacheInvalidatableRequest for EntraGroupsForMemberRequest {
    fn invalidate<'a>(&'a self, executor: &'a CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()> {
        Box::pin(async move {
            // Group identities are discovered dynamically. Refresh that discovery
            // first, then invalidate exactly the batch that execution will consume.
            let group_ids = EntraMemberGroupIdsRequest {
                principal_id: self.principal_id,
                authentication: self.authentication,
            }
            .execute(executor)
            .with_invalidation(true)
            .await?;
            self.group_lookup_request(&group_ids.value)
                .invalidate(executor)
                .await
        })
    }
}

impl ExecutableRequest for EntraGroupsForMemberRequest {
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
            let group_ids = EntraMemberGroupIdsRequest {
                principal_id: self.principal_id,
                authentication: self.authentication,
            }
            .execute(&executor)
            .await?;
            let groups = self
                .group_lookup_request(&group_ids.value)
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

cloud_terrastodon_registry::register_thing!(EntraGroupsForMemberRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupsForMemberRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraGroupsForMemberRequest => Vec<Arc<EntraGroup>>,
    effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraGroupsForMemberRequest,
    name = "entra-groups-for-member",
    cached
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membership_order_and_duplicates_share_the_refreshed_lookup_scope() -> eyre::Result<()> {
        let request = EntraGroupsForMemberRequest {
            principal_id: "11111111-1111-1111-1111-111111111111".parse()?,
            authentication: crate::test_authentication::test_authentication(),
        };
        let first: EntraGroupId = "22222222-2222-2222-2222-222222222222".parse()?;
        let second: EntraGroupId = "33333333-3333-3333-3333-333333333333".parse()?;
        let lookup = request.group_lookup_request(&[second, first, second]);
        assert_eq!(lookup.group_ids, vec![first, second]);
        assert_eq!(
            lookup.cache_key()?,
            request.group_lookup_request(&[first, second]).cache_key()?
        );
        Ok(())
    }
}
