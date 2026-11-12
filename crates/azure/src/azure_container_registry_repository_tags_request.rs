use crate::container_registry_pagination::next_registry_page;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ContainerRegistryEndpoint;
use cloud_terrastodon_azure_types::ContainerRegistryRepositoryName;
use cloud_terrastodon_azure_types::ContainerRegistryRepositoryTag;
use cloud_terrastodon_azure_types::ContainerRegistryTagListResponse;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::ensure;
use facet::Facet;
use http::Method;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

/// List all repository tags at a resolved endpoint with a scoped ACR bearer exchange.
/// Resolve an endpoint separately or reuse [`ContainerRegistryEndpoint::try_from`]
/// on an existing registry; this operation makes no Resource Graph lookup.
/// [REST API 2021-07-01](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/container-registry/get-tags?view=rest-registry-dataplane-2021-07-01).
#[derive(Facet)]
pub struct AzureContainerRegistryRepositoryTagsRequest<'a> {
    pub endpoint: Cow<'a, ContainerRegistryEndpoint>,
    pub repository_name: Cow<'a, ContainerRegistryRepositoryName>,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureContainerRegistryRepositoryTagsRequest<'_> {}

impl HasCacheKey for AzureContainerRegistryRepositoryTagsRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(
            PathBuf::from("rest/container-registry/tenants")
                .join(self.authentication.tenant_id.to_string())
                .join("login-servers")
                .join(self.endpoint.login_server.as_str())
                .join("repositories")
                .join("by-name")
                .join(sanitize_windows_path_component(&self.repository_name))
                .join("tags"),
        )
    }
}

impl ExecutableRequest for AzureContainerRegistryRepositoryTagsRequest<'_> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<ContainerRegistryRepositoryTag>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            ensure!(
                self.repository_name
                    .split('/')
                    .all(|segment| !segment.is_empty() && segment != "." && segment != ".."),
                "repository name contains an ambiguous URL path segment"
            );
            let mut url = self.endpoint.login_server.endpoint()?;
            {
                let mut path = url
                    .path_segments_mut()
                    .map_err(|_| eyre::eyre!("registry endpoint cannot contain path segments"))?;
                path.clear().extend(["acr", "v1"]);
                for segment in self.repository_name.split('/') {
                    path.push(segment);
                }
                path.push("_tags");
            }
            url.query_pairs_mut()
                .append_pair("api-version", "2021-07-01")
                .append_pair("n", "100");
            let cache = self.cache_key()?;
            let mut visited = HashSet::new();
            let mut items = Vec::new();
            loop {
                executor.cancellation().bail_if_cancelled()?;
                ensure!(
                    visited.insert(url.clone()),
                    "registry returned a repeated continuation URL"
                );
                let current = url.clone();
                let (response, next) = RestRequest::from_method_and_url(Method::GET, url.clone())?
                    .authentication(self.authentication.into())
                    .cache(CacheKey {
                        scope: cache
                            .scope
                            .join("pages")?
                            .join(blake3::hash(url.as_str().as_bytes()).to_hex().as_str())?,
                        valid_for: cache.valid_for,
                    })
                    .receive_with_mapper_cached::<ContainerRegistryTagListResponse, _, _>(
                        move |response, metadata| {
                            let next = next_registry_page(
                                &current,
                                metadata.header("link"),
                                response.link.as_deref(),
                            )?;
                            Ok((response, next))
                        },
                        &executor,
                        CacheOperationOwner::of::<
                            AzureContainerRegistryRepositoryTagsRequest<'static>,
                        >(),
                    )
                    .await?;
                items.extend(response.tags.iter().cloned());
                match next {
                    Some(next) => url = next,
                    None => return Ok(items),
                }
            }
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureContainerRegistryRepositoryTagsRequest<'static>);

impl<'a> Arbitrary<'a> for AzureContainerRegistryRepositoryTagsRequest<'static> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            endpoint: Cow::Owned(Arbitrary::arbitrary(u)?),
            repository_name: Cow::Owned(Arbitrary::arbitrary(u)?),
            authentication: Arbitrary::arbitrary(u)?,
        })
    }
}
cloud_terrastodon_registry::register_arbitrary!(
    AzureContainerRegistryRepositoryTagsRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(AzureContainerRegistryRepositoryTagsRequest<'static> => Vec<Arc<ContainerRegistryRepositoryTag>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureContainerRegistryRepositoryTagsRequest<'static>,
    name = "container-registry-repository-tags",
    cached
);
