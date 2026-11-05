use crate::container_registry_pagination::next_registry_page;
use crate::fetch_container_registry_endpoint;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::ContainerRegistryId;
use cloud_terrastodon_azure_types::ContainerRegistryRepositoryListResponse;
use cloud_terrastodon_azure_types::ContainerRegistryRepositoryName;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::ensure;
use http::Method;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;

/// List all registry repositories, using its real login server and a scoped ACR bearer exchange.
/// [REST API 2021-07-01](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/container-registry/get-repositories?view=rest-registry-dataplane-2021-07-01).
#[derive(facet::Facet)]
pub struct ContainerRegistryRepositoryListRequest<'a> {
    pub registry_id: Cow<'a, ContainerRegistryId>,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
}
pub fn fetch_container_registry_repository_names<'a>(
    registry_id: &'a ContainerRegistryId,
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> ContainerRegistryRepositoryListRequest<'a> {
    ContainerRegistryRepositoryListRequest {
        registry_id: Cow::Borrowed(registry_id),
        auth_context: Cow::Borrowed(auth_context),
        cancellation: cancellation.clone(),
    }
}
#[async_trait]
impl CacheableCommand for ContainerRegistryRepositoryListRequest<'_> {
    type Output = Vec<ContainerRegistryRepositoryName>;
    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
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
    async fn run(self) -> Result<Self::Output> {
        let endpoint = fetch_container_registry_endpoint(
            &self.registry_id,
            &self.auth_context,
            &self.cancellation,
        )
        .await?;
        let mut url = endpoint.login_server.endpoint()?.join("/acr/v1/_catalog")?;
        url.query_pairs_mut()
            .append_pair("api-version", "2021-07-01")
            .append_pair("n", "100");
        let cache = self.cache_key();
        let mut visited = HashSet::new();
        let mut items = Vec::new();
        loop {
            self.cancellation.bail_if_cancelled()?;
            ensure!(
                visited.insert(url.clone()),
                "registry returned a repeated continuation URL"
            );
            let current = url.clone();
            let (response, next) =
                RestRequest::from_method_and_url(Method::GET, url.clone(), &self.cancellation)?
                    .azure_auth_context(&self.auth_context)
                    .cache(CacheKey {
                        path: cache
                            .path
                            .join("pages")
                            .join(blake3::hash(url.as_str().as_bytes()).to_hex().as_str()),
                        valid_for: cache.valid_for,
                    })
                    .receive_with_mapper::<ContainerRegistryRepositoryListResponse, _, _>(
                        move |response, metadata| {
                            let next = next_registry_page(
                                &current,
                                metadata.header("link"),
                                response.link.as_deref(),
                            )?;
                            Ok((response, next))
                        },
                    )
                    .await?;
            items.extend(response.repositories);
            match next {
                Some(next) => url = next,
                None => return Ok(items),
            }
        }
    }
}
cloud_terrastodon_command::impl_cacheable_into_future!(ContainerRegistryRepositoryListRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(ContainerRegistryRepositoryListRequest<'static>);

impl<'a> arbitrary::Arbitrary<'a> for ContainerRegistryRepositoryListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            registry_id: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),

            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
            cancellation: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistryRepositoryListRequest<'static>);
cloud_terrastodon_registry_execution::register_into_future!(ContainerRegistryRepositoryListRequest<'static> => Vec<ContainerRegistryRepositoryName>, effects = [Read]);
