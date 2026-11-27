use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraDirectoryObjectGroup;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// List groups directly owned by a user through Microsoft Graph v1.0
/// [ownedObjects](https://learn.microsoft.com/en-us/graph/api/user-list-ownedobjects?view=graph-rest-1.0).
///
/// Uses the group OData cast exposed by [Get-MgUserOwnedObjectAsGroup](https://learn.microsoft.com/en-us/powershell/module/microsoft.graph.users/get-mguserownedobjectasgroup).
/// The advanced query is eventually consistent. Objects with limited information
/// retain the group ID even when its display name is unavailable.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraGroupListByOwnerRequest {
    pub user_id: EntraUserId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraGroupListByOwnerRequest {}

impl HasCacheKey for EntraGroupListByOwnerRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "ms",
                "graph",
                "v1.0",
                "GET",
                "tenants",
                &self.authentication.tenant_id.to_string(),
                "users",
                &self.user_id.to_string(),
                "owned-groups",
            ])?,
            valid_for: Duration::from_secs(60),
        })
    }
}

impl ExecutableRequest for EntraGroupListByOwnerRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraDirectoryObjectGroup>>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let mut url = Url::parse(&format!(
                "https://graph.microsoft.com/v1.0/users/{}/ownedObjects/microsoft.graph.group",
                self.user_id,
            ))?;
            url.query_pairs_mut()
                .append_pair("$select", "id,displayName")
                .append_pair("$count", "true");
            MicrosoftGraphHelper::new(url, Some(self.cache_key()?), self.authentication)
                .with_consistency_level_eventual()?
                .fetch_all(&executor)
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupListByOwnerRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupListByOwnerRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraGroupListByOwnerRequest => Vec<Arc<EntraDirectoryObjectGroup>>,
    effects = [Read]
);
