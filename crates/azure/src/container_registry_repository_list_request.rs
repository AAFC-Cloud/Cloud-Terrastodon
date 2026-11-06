use crate::container_registry_pagination::next_registry_page;
use crate::fetch_container_registry_endpoint;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::ContainerRegistryId;
use cloud_terrastodon_azure_types::ContainerRegistryRepositoryListResponse;
use cloud_terrastodon_azure_types::ContainerRegistryRepositoryName;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
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
pub struct ContainerRegistryRepositoryListRequest<'a> {
    pub registry_id: Cow<'a, ContainerRegistryId>,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}
#[track_caller]
pub fn fetch_container_registry_repository_names<'a>(
    registry_id: &'a ContainerRegistryId,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, ContainerRegistryRepositoryListRequest<'a>> {
    executor.execute(ContainerRegistryRepositoryListRequest {
        registry_id: Cow::Borrowed(registry_id),
        auth_context: Cow::Borrowed(auth_context),
    })
}
impl InvalidatesOwnCacheKey for ContainerRegistryRepositoryListRequest<'_> {}

impl HasCacheKey for ContainerRegistryRepositoryListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(
            PathBuf::from("rest/container-registry/tenants")
                .join(self.auth_context.tenant_id.to_string())
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

impl ExecutableRequest for ContainerRegistryRepositoryListRequest<'_> {
    type Error = eyre::Report;
    type Output = Vec<Arc<ContainerRegistryRepositoryName>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let endpoint =
                fetch_container_registry_endpoint(&self.registry_id, &self.auth_context, &executor)
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
                    .azure_auth_context(&self.auth_context)?
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
                        CacheOperationOwner::of::<ContainerRegistryRepositoryListRequest<'static>>(
                        ),
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

cloud_terrastodon_registry::register_thing!(ContainerRegistryRepositoryListRequest<'static>);

impl<'a> arbitrary::Arbitrary<'a> for ContainerRegistryRepositoryListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            registry_id: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),

            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistryRepositoryListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(ContainerRegistryRepositoryListRequest<'static> => Vec<Arc<ContainerRegistryRepositoryName>>, effects = [Read]);
