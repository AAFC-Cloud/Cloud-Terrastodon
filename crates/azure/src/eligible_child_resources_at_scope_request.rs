use crate::FetchChildrenBehaviour;
use cloud_terrastodon_auth::AzureTenantAuthContext;
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
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

/// List eligible children of one ARM scope.
/// [Authorization 2020-10-01](https://learn.microsoft.com/rest/api/authorization/eligible-child-resources/get?view=rest-authorization-2020-10-01).
#[derive(facet::Facet)]
pub struct EligibleChildResourcesAtScopeRequest<'a> {
    pub scope: ScopeImpl,
    pub behaviour: FetchChildrenBehaviour,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}
impl<'a> arbitrary::Arbitrary<'a> for EligibleChildResourcesAtScopeRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            scope: arbitrary::Arbitrary::arbitrary(u)?,
            behaviour: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}
impl HasCacheKey for EligibleChildResourcesAtScopeRequest<'_> {
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
impl InvalidatesOwnCacheKey for EligibleChildResourcesAtScopeRequest<'_> {}
impl ExecutableRequest for EligibleChildResourcesAtScopeRequest<'_> {
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
                    .azure_auth_context(&self.auth_context)?
                    .cache(self.cache_key()?.join(page_index.to_string())?)
                    .receive_cached::<EligibleChildResourceListResponse>(
                        &executor,
                        CacheOperationOwner::of::<EligibleChildResourcesAtScopeRequest<'static>>(),
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
cloud_terrastodon_registry::register_thing!(EligibleChildResourcesAtScopeRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EligibleChildResourcesAtScopeRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(EligibleChildResourcesAtScopeRequest<'static> => Vec<Arc<EligibleChildResource>>, effects = [Read]);
