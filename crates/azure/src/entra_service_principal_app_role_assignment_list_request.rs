use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraAppRoleAssignment;
use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

/// List [application permissions granted to a service principal](https://learn.microsoft.com/graph/api/serviceprincipal-list-approleassignments?view=graph-rest-1.0).
///
/// These are outbound, app-only permissions granted to the client service
/// principal. The read supports Graph scopes such as `Application.Read.All`
/// and `Directory.Read.All`.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraServicePrincipalAppRoleAssignmentListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub service_principal_id: EntraServicePrincipalObjectId,
}

impl InvalidatesOwnCacheKey for EntraServicePrincipalAppRoleAssignmentListRequest {}

impl HasCacheKey for EntraServicePrincipalAppRoleAssignmentListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service-principals",
            self.authentication.tenant_id.to_string().as_str(),
            self.service_principal_id.to_string().as_str(),
            "app-role-assignments",
        ]))
    }
}

impl ExecutableRequest for EntraServicePrincipalAppRoleAssignmentListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraAppRoleAssignment>>;

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
                "https://graph.microsoft.com/v1.0/servicePrincipals/{}/appRoleAssignments",
                self.service_principal_id
            );
            MicrosoftGraphHelper::new(url, Some(self.cache_key()?), self.authentication)
                .fetch_all(&executor)
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraServicePrincipalAppRoleAssignmentListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraServicePrincipalAppRoleAssignmentListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraServicePrincipalAppRoleAssignmentListRequest => Vec<Arc<EntraAppRoleAssignment>>,
    effects = [Read]
);
