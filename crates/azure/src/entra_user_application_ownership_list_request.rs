use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraOwnedApplicationRegistration;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

/// List [app registrations directly owned by a user](https://learn.microsoft.com/graph/api/user-list-ownedobjects?view=graph-rest-1.0).
///
/// Graph sometimes returns just an object ID when the caller can read the
/// relationship but lacks access to the app's details.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraUserApplicationOwnershipListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub user_id: EntraUserId,
}

impl InvalidatesOwnCacheKey for EntraUserApplicationOwnershipListRequest {}

impl HasCacheKey for EntraUserApplicationOwnershipListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "users",
            self.authentication.tenant_id.to_string().as_str(),
            self.user_id.to_string().as_str(),
            "owned-application-registrations",
        ]))
    }
}

impl ExecutableRequest for EntraUserApplicationOwnershipListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraOwnedApplicationRegistration>>;

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
            let url = format!(
                "https://graph.microsoft.com/v1.0/users/{}/ownedObjects/microsoft.graph.application?$select=id,appId,displayName&$count=true",
                self.user_id
            );
            MicrosoftGraphHelper::new(url, Some(self.cache_key()?), self.authentication)
                .with_consistency_level_eventual()?
                .fetch_all(&executor)
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraUserApplicationOwnershipListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUserApplicationOwnershipListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraUserApplicationOwnershipListRequest => Vec<Arc<EntraOwnedApplicationRegistration>>,
    effects = [Read]
);
