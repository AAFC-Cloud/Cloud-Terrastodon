use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraPimCollectionResponse;
use cloud_terrastodon_azure_types::PimEntraRoleDefinition;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
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

/// List the tenant's legacy [PIM role definitions](https://learn.microsoft.com/graph/api/resources/governanceroledefinition?view=graph-rest-beta).
#[derive(Arbitrary, Facet)]
pub struct EntraPimRoleDefinitionListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl HasCacheKey for EntraPimRoleDefinitionListRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "graph",
                "beta",
                "tenants",
                &self.authentication.tenant_id.to_string(),
                "pim",
                "role-definitions",
            ])?,
            valid_for: Duration::from_secs(3600),
        })
    }
}
impl InvalidatesOwnCacheKey for EntraPimRoleDefinitionListRequest {}
impl ExecutableRequest for EntraPimRoleDefinitionListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<PimEntraRoleDefinition>>;
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let initial_url = Url::parse(&format!(
                "https://graph.microsoft.com/beta/privilegedAccess/aadroles/resources/{}/roleDefinitions?$select=id,displayName,type,isbuiltIn&$orderby=displayName",
                self.authentication.tenant_id
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
                    .receive_cached::<EntraPimCollectionResponse<PimEntraRoleDefinition>>(
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
cloud_terrastodon_registry::register_thing!(EntraPimRoleDefinitionListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraPimRoleDefinitionListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraPimRoleDefinitionListRequest => Vec<Arc<PimEntraRoleDefinition>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraPimRoleDefinitionListRequest,
    name = "pim-entra-role-definition-list",
    cached
);
