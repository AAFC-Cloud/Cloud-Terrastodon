use crate::MicrosoftGraphBatchRequestEntry;
use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraDirectoryObject;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// List a group's direct members as shallow Microsoft Graph directory objects.
///
/// Follows every `@odata.nextLink` beneath this tenant/group's invalidatable
/// cache key. Optional names retain limited-information responses. Graph v1.0
/// [list members](https://learn.microsoft.com/en-us/graph/api/group-list-members?view=graph-rest-1.0)
/// can omit service principals, so pagination alone does not prove completeness.
#[derive(Facet)]
pub struct EntraGroupMembersListRequest {
    pub group_id: EntraGroupId,
    pub authentication: AzureBearerAuthenticationSelection,
}
impl EntraGroupMembersListRequest {
    pub fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/groups/{}/members",
            self.group_id
        )
    }
}
impl From<EntraGroupMembersListRequest>
    for MicrosoftGraphBatchRequestEntry<Vec<Arc<EntraDirectoryObject>>>
{
    fn from(request: EntraGroupMembersListRequest) -> Self {
        MicrosoftGraphBatchRequestEntry::new_get(
            format!("group-members-for-{}", request.group_id),
            request.url(),
        )
    }
}

impl<'a> arbitrary::Arbitrary<'a> for EntraGroupMembersListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            group_id: arbitrary::Arbitrary::arbitrary(u)?,
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for EntraGroupMembersListRequest {}

impl HasCacheKey for EntraGroupMembersListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "v1.0",
            "GET",
            "tenants",
            self.authentication.tenant_id.to_string().as_str(),
            "groups",
            self.group_id.to_string().as_str(),
            "members",
            "directory_objects",
        ]))
    }
}

impl ExecutableRequest for EntraGroupMembersListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraDirectoryObject>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(tenant_id = %self.authentication.tenant_id, group_id = %self.group_id, "Fetching group members");
            let query = MicrosoftGraphHelper::new(
                format!(
                    "https://graph.microsoft.com/v1.0/groups/{}/members",
                    self.group_id
                ),
                Some(self.cache_key()?),
                self.authentication,
            );
            let members = query.fetch_all::<EntraDirectoryObject>(&executor).await?;
            debug!(
                "Found {} members for group {}",
                members.len(),
                self.group_id
            );
            Ok(members)
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupMembersListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupMembersListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupMembersListRequest => Vec<Arc<EntraDirectoryObject>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraGroupMembersListRequest,
    name = "entra-group-members-list",
    cached
);
