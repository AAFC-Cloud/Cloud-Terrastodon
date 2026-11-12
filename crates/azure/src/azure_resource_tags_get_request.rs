use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureResourceTags;
use cloud_terrastodon_azure_types::ResourceTagsId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use http::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

/// Read the tags on one explicitly selected ARM resource or subscription.
/// [Tags Get At Scope](https://learn.microsoft.com/rest/api/resources/tags/get-at-scope?view=rest-resources-2021-04-01).
/// Uses the unchanged [2022-09-01 contract](https://github.com/Azure/azure-rest-api-specs/blob/main/specification/resources/resource-manager/Microsoft.Resources/resources/stable/2022-09-01/resources.json).
#[must_use = "Execute this request with an executor"]
#[derive(Debug, Facet)]
pub struct AzureResourceTagsGetRequest<'a> {
    pub authentication: AzureBearerAuthenticationSelection,
    pub id: Cow<'a, ResourceTagsId>,
}
impl<'a> Arbitrary<'a> for AzureResourceTagsGetRequest<'static> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: Arbitrary::arbitrary(u)?,
            id: Cow::Owned(Arbitrary::arbitrary(u)?),
        })
    }
}

impl HasCacheKey for AzureResourceTagsGetRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let mut path = PathBuf::from("rest/arm/tenants")
            .join(self.authentication.tenant_id.to_string())
            .join("resource-tags");
        for component in self
            .id
            .expanded_form()
            .to_ascii_lowercase()
            .trim_start_matches('/')
            .split('/')
        {
            path.push(sanitize_windows_path_component(component));
        }
        CacheKey::from_relative_path(path.join("get/api-version/2022-09-01"))
    }
}
impl InvalidatesOwnCacheKey for AzureResourceTagsGetRequest<'_> {}
impl ExecutableRequest for AzureResourceTagsGetRequest<'_> {
    type Output = Arc<AzureResourceTags>;
    type Error = eyre::Report;
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let key = self.cache_key()?;
            let mut url = Url::parse("https://management.azure.com")?;
            // ARM scope identity is case-insensitive; use the same spelling in
            // the URL fingerprint and cache key.
            url.set_path(&self.id.expanded_form().to_ascii_lowercase());
            url.query_pairs_mut()
                .append_pair("api-version", "2022-09-01");
            RestRequest::from_method_and_url(Method::GET, url)?
                .authentication(self.authentication.into())
                .cache(key)
                .receive_with_validator_cached(
                    move |response: &AzureResourceTags| response.validate_identity(&self.id),
                    &executor,
                    CacheOperationOwner::of::<AzureResourceTagsGetRequest<'static>>(),
                )
                .await
        })
    }
}
cloud_terrastodon_registry::register_thing!(AzureResourceTagsGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceTagsGetRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureResourceTagsGetRequest<'static> => Arc<AzureResourceTags>, effects = [Read]);
cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureResourceTagsGetRequest<'static>,
    name = "resource-tags",
    cached
);
