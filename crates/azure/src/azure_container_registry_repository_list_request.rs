use crate::AzureContainerRegistryEndpointRequest;
use crate::container_registry_pagination::next_registry_page;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ContainerRegistryId;
use cloud_terrastodon_azure_types::ContainerRegistryRepositoryListResponse;
use cloud_terrastodon_azure_types::ContainerRegistryRepositoryName;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
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

/// List all registry repositories, using its real login server and a scoped ACR bearer exchange.
/// [REST API 2021-07-01](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/container-registry/get-repositories?view=rest-registry-dataplane-2021-07-01).
#[derive(Facet)]
pub struct AzureContainerRegistryRepositoryListRequest<'a> {
    pub registry_id: Cow<'a, ContainerRegistryId>,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl CacheInvalidationKeys for AzureContainerRegistryRepositoryListRequest<'_> {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![
            self.cache_key()?,
            AzureContainerRegistryEndpointRequest {
                registry_id: Cow::Borrowed(&self.registry_id),
                authentication: self.authentication,
            }
            .cache_key()?,
        ])
    }
}

impl HasCacheKey for AzureContainerRegistryRepositoryListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(
            PathBuf::from("rest/container-registry/tenants")
                .join(self.authentication.tenant_id.to_string())
                .join("subscriptions")
                .join(
                    self.registry_id
                        .resource_group_id
                        .subscription_id
                        .to_string(),
                )
                .join("resource-groups")
                .join(sanitize_windows_path_component(
                    self.registry_id
                        .resource_group_id
                        .resource_group_name
                        .as_str(),
                ))
                .join("registries")
                .join(self.registry_id.container_registry_name.as_str())
                .join("repositories"),
        )
    }
}

impl ExecutableRequest for AzureContainerRegistryRepositoryListRequest<'_> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<ContainerRegistryRepositoryName>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let endpoint = AzureContainerRegistryEndpointRequest {
                registry_id: Cow::Borrowed(&self.registry_id),
                authentication: self.authentication,
            }
            .execute(&executor)
            .await?;
            let mut url = endpoint.login_server.endpoint()?.join("/acr/v1/_catalog")?;
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
                    .receive_with_mapper_cached::<ContainerRegistryRepositoryListResponse, _, _>(
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
                            AzureContainerRegistryRepositoryListRequest<'static>,
                        >(),
                    )
                    .await?;
                items.extend(response.repositories.iter().cloned());
                match next {
                    Some(next) => url = next,
                    None => return Ok(items),
                }
            }
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureContainerRegistryRepositoryListRequest<'static>);

impl<'a> arbitrary::Arbitrary<'a> for AzureContainerRegistryRepositoryListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            registry_id: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),

            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}
cloud_terrastodon_registry::register_arbitrary!(
    AzureContainerRegistryRepositoryListRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(AzureContainerRegistryRepositoryListRequest<'static> => Vec<Arc<ContainerRegistryRepositoryName>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureContainerRegistryRepositoryListRequest<'static>,
    name = "container-registry-repository-list",
    cached
);
