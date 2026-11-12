use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::MicrosoftGraphResponse;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use http::Method;
use std::path::PathBuf;
use std::sync::Arc;

/// List transitive group IDs for a directory object, including non-security groups.
/// [Microsoft Graph v1.0 getMemberGroups](https://learn.microsoft.com/graph/api/directoryobject-getmembergroups?view=graph-rest-1.0).
/// Graph reports an error when its result-size limit is exceeded.
#[derive(Arbitrary, Facet)]
pub struct EntraGroupIdListByMemberRequest {
    pub principal_id: EntraPrincipalId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl HasCacheKey for EntraGroupIdListByMemberRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "POST".to_string(),
            "entra_member_group_ids".to_string(),
            self.authentication.tenant_id.to_string(),
            self.principal_id.to_string(),
        ]))
    }
}

impl InvalidatesOwnCacheKey for EntraGroupIdListByMemberRequest {}

impl ExecutableRequest for EntraGroupIdListByMemberRequest {
    type Error = eyre::Report;
    type Output = Arc<MicrosoftGraphResponse<EntraGroupId>>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            RestRequest::new(
                Method::POST,
                format!(
                    "https://graph.microsoft.com/v1.0/directoryObjects/{}/getMemberGroups",
                    self.principal_id
                ),
            )?
            // This POST is an observation, with no directory mutation.
            .cancellation_safe()
            .authentication(self.authentication.into())
            .body("{\"securityEnabledOnly\":false}")
            .cache(self.cache_key()?)
            .receive_cached(&executor, CacheOperationOwner::of::<Self>())
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupIdListByMemberRequest);
cloud_terrastodon_registry::register_thing!(MicrosoftGraphResponse<EntraGroupId>);
cloud_terrastodon_registry::register_arbitrary!(MicrosoftGraphResponse<EntraGroupId>);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupIdListByMemberRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraGroupIdListByMemberRequest => Arc<MicrosoftGraphResponse<EntraGroupId>>,
    effects = [Read]
);
cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraGroupIdListByMemberRequest,
    name = "entra-group-id-list-by-member",
    cached
);
