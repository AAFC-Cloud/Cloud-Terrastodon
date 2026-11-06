use crate::fetch_groups_by_id;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use http::Method;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct EntraGroupsForMemberRequest<'a> {
    pub principal_id: EntraPrincipalId,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraGroupsForMemberRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            principal_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_entra_groups_for_member<'a>(
    principal_id: EntraPrincipalId,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EntraGroupsForMemberRequest<'a>> {
    executor.execute(EntraGroupsForMemberRequest {
        principal_id,
        auth_context: Cow::Borrowed(auth_context),
    })
}

#[derive(facet::Facet)]
struct GetMemberGroupsResponse {
    value: Vec<EntraGroupId>,
}

impl InvalidatesOwnCacheKey for EntraGroupsForMemberRequest<'_> {}

impl HasCacheKey for EntraGroupsForMemberRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "POST".to_string(),
            "entra_groups_for_member".to_string(),
            self.auth_context.tenant_id.to_string(),
            self.principal_id.to_string(),
        ]))
    }
}

impl ExecutableRequest for EntraGroupsForMemberRequest<'_> {
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraGroup>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                tenant_id = %self.auth_context.tenant_id,
                principal_id = %self.principal_id,
                "Fetching Entra groups for principal"
            );
            let response = RestRequest::new(
                Method::POST,
                format!(
                    "https://graph.microsoft.com/v1.0/directoryObjects/{}/getMemberGroups",
                    self.principal_id
                ),
            )?
            .cancellation_safe()
            .azure_auth_context(&self.auth_context)?
            .body("{\"securityEnabledOnly\":false}")
            .receive_cached::<GetMemberGroupsResponse>(
                &executor,
                CacheOperationOwner::of::<EntraGroupsForMemberRequest<'static>>(),
            )
            .await?;

            let group_ids: HashSet<_> = response.value.iter().copied().collect();
            let groups =
                fetch_groups_by_id(group_ids, self.auth_context.as_ref(), &executor).await?;
            debug!(
                tenant_id = %self.auth_context.tenant_id,
                principal_id = %self.principal_id,
                count = groups.len(),
                "Found Entra groups for principal"
            );
            Ok(groups)
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupsForMemberRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupsForMemberRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraGroupsForMemberRequest<'static> => Vec<Arc<EntraGroup>>,
    effects = [Read]
);
