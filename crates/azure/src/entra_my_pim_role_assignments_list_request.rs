use crate::EntraCurrentUserRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraPimCollectionResponse;
use cloud_terrastodon_azure_types::GovernanceRoleAssignment;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use http::Method;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// List the current delegated user's eligible and active [PIM assignments](https://learn.microsoft.com/graph/api/resources/governanceroleassignment?view=graph-rest-beta).
#[derive(Arbitrary, Facet)]
pub struct EntraMyPimRoleAssignmentsListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl HasCacheKey for EntraMyPimRoleAssignmentsListRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "graph",
                "beta",
                "tenants",
                &self.authentication.tenant_id.to_string(),
                "pim",
                "me",
                "role-assignments",
            ])?,
            valid_for: Duration::from_secs(3600),
        })
    }
}
impl CacheInvalidationKeys for EntraMyPimRoleAssignmentsListRequest {
    fn cache_keys_to_invalidate(&self) -> Result<Vec<CacheKey>> {
        Ok(vec![
            self.cache_key()?,
            EntraCurrentUserRequest {
                authentication: self.authentication,
            }
            .cache_key()?,
        ])
    }
}
impl ExecutableRequest for EntraMyPimRoleAssignmentsListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<GovernanceRoleAssignment>>;
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let principal = EntraCurrentUserRequest {
                authentication: self.authentication,
            }
            .execute(&executor)
            .await?;
            let initial_url = Url::parse(&format!(
                "https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignments?$filter=(subject/id eq '{}') and (assignmentState in ('Eligible', 'Active'))&$select=assignmentState,endDateTime,id,linkedEligibleRoleAssignmentId,memberType,roleDefinitionId,startDateTime,status,subjectId",
                principal.id,
            ))?;
            let mut next = Some(initial_url.clone());
            let mut visited = std::collections::HashSet::new();
            let mut values = Vec::new();
            let mut page_index = 0usize;
            while let Some(url) = next {
                executor.cancellation().bail_if_cancelled()?;
                eyre::ensure!(
                    url.origin() == initial_url.origin()
                        && url.path() == initial_url.path()
                        && url.username().is_empty()
                        && url.password().is_none()
                        && url.fragment().is_none(),
                    "Graph PIM continuation left the selected resource"
                );
                eyre::ensure!(
                    visited.insert(url.clone()),
                    "Graph PIM continuation repeated a page"
                );
                let request = RestRequest::from_method_and_url(Method::GET, url)?
                    .authentication(self.authentication.into())
                    .cache(self.cache_key()?.join(format!("pages/{page_index}"))?);
                let mut result = request
                    .clone()
                    .receive_cached::<EntraPimCollectionResponse<GovernanceRoleAssignment>>(
                        &executor,
                        CacheOperationOwner::of::<Self>(),
                    )
                    .await;
                if result.is_err() {
                    executor.cancellation().bail_if_cancelled()?;
                    // Preserve the existing single retry for transient gateway errors.
                    result = request
                        .receive_cached(&executor, CacheOperationOwner::of::<Self>())
                        .await;
                }
                let response = result?;
                values.extend(response.value.iter().cloned());
                next = response.next_link.clone();
                page_index += 1;
            }
            Ok(values)
        })
    }
}
cloud_terrastodon_registry::register_thing!(EntraMyPimRoleAssignmentsListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraMyPimRoleAssignmentsListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraMyPimRoleAssignmentsListRequest => Vec<Arc<GovernanceRoleAssignment>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraMyPimRoleAssignmentsListRequest,
    name = "my-entra-pim-role-assignments-list",
    cached
);
