use crate::FetchChildrenBehaviour;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EligibleChildResource;
use cloud_terrastodon_azure_types::EligibleChildResourceListResponse;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_azure_types::ScopeImpl;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use std::sync::Arc;
use std::time::Duration;

/// List eligible children of one ARM scope.
/// [Authorization 2020-10-01](https://learn.microsoft.com/rest/api/authorization/eligible-child-resources/get?view=rest-authorization-2020-10-01).
#[derive(Arbitrary, Facet)]
pub struct AzureEligibleChildResourcesAtScopeRequest {
    pub scope: ScopeImpl,
    pub behaviour: FetchChildrenBehaviour,
    pub authentication: AzureBearerAuthenticationSelection,
}
impl HasCacheKey for AzureEligibleChildResourcesAtScopeRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let mut components = vec![
            "az".to_string(),
            "rest".to_string(),
            "GET".to_string(),
            "eligibleChildResources".to_string(),
        ];
        components.extend(
            self.scope
                .expanded_form()
                .split('/')
                .filter(|component| !component.is_empty())
                .map(str::to_owned),
        );
        Ok(CacheKey {
            scope: CacheScope::try_from_components(components)?,
            valid_for: Duration::MAX,
        })
    }
}
impl InvalidatesOwnCacheKey for AzureEligibleChildResourcesAtScopeRequest {}
impl ExecutableRequest for AzureEligibleChildResourcesAtScopeRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<EligibleChildResource>>;
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let mut next_url = Some(url::Url::parse(&format!(
                "https://management.azure.com{}/providers/Microsoft.Authorization/eligibleChildResources?api-version=2020-10-01",
                self.scope.expanded_form()
            ))?);
            if self.behaviour == FetchChildrenBehaviour::GetAllChildren
                && let Some(url) = next_url.as_mut()
            {
                url.query_pairs_mut().append_pair("getAllChildren", "true");
            }
            let mut page_index = 0;
            let mut resources = Vec::new();
            while let Some(url) = next_url.take() {
                executor.cancellation().bail_if_cancelled()?;
                let page = RestRequest::from_method_and_url(http::Method::GET, url)?
                    .authentication(self.authentication.into())
                    .cache(self.cache_key()?.join(page_index.to_string())?)
                    .receive_cached::<EligibleChildResourceListResponse>(
                        &executor,
                        CacheOperationOwner::of::<AzureEligibleChildResourcesAtScopeRequest>(),
                    )
                    .await?;
                resources.extend(page.value.iter().cloned());
                next_url = page.next_link.clone();
                page_index += 1;
            }
            Ok(resources)
        })
    }
}
cloud_terrastodon_registry::register_thing!(AzureEligibleChildResourcesAtScopeRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureEligibleChildResourcesAtScopeRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureEligibleChildResourcesAtScopeRequest => Vec<Arc<EligibleChildResource>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureEligibleChildResourcesAtScopeRequest,
    name = "eligible-child-resources-at-scope",
    cached
);
