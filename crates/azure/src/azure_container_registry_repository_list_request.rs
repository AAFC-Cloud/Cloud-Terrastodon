use crate::container_registry_pagination::next_registry_page;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ContainerRegistryEndpoint;
use cloud_terrastodon_azure_types::ContainerRegistryRepositoryListResponse;
use cloud_terrastodon_azure_types::ContainerRegistryRepositoryName;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use eyre::ensure;
use facet::Facet;
use http::Method;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

/// List all repositories at a resolved registry endpoint with a scoped ACR bearer exchange.
/// Resolve an endpoint separately or reuse [`ContainerRegistryEndpoint::try_from`]
/// on an existing registry; this operation makes no Resource Graph lookup.
/// [REST API 2021-07-01](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/container-registry/get-repositories?view=rest-registry-dataplane-2021-07-01).
#[derive(Facet)]
pub struct AzureContainerRegistryRepositoryListRequest<'a> {
    pub endpoint: Cow<'a, ContainerRegistryEndpoint>,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureContainerRegistryRepositoryListRequest<'_> {}

impl HasCacheKey for AzureContainerRegistryRepositoryListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(
            PathBuf::from("rest/container-registry/tenants")
                .join(self.authentication.tenant_id.to_string())
                .join("login-servers")
                .join(self.endpoint.login_server.as_str())
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
            let mut url = self
                .endpoint
                .login_server
                .endpoint()?
                .join("/acr/v1/_catalog")?;
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

impl<'a> Arbitrary<'a> for AzureContainerRegistryRepositoryListRequest<'static> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            endpoint: Cow::Owned(Arbitrary::arbitrary(u)?),
            authentication: Arbitrary::arbitrary(u)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AzureContainerRegistryRepositoryTagsRequest;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_cache::CacheInvalidationKeys;

    #[test]
    fn data_cache_follows_the_selected_login_server_and_repository() -> eyre::Result<()> {
        let authentication = AzureBearerAuthenticationSelection {
            account_id: "11111111-1111-1111-1111-111111111111".parse()?,
            auth_source: AzureBearerAuthSource::AzureCli,
            tenant_id: "22222222-2222-2222-2222-222222222222".parse()?,
        };
        let endpoint = ContainerRegistryEndpoint {
            id: "/subscriptions/33333333-3333-3333-3333-333333333333/resourceGroups/fixture/providers/Microsoft.ContainerRegistry/registries/fixture".parse()?,
            login_server: "fixture-ab12.azurecr.io".parse()?,
        };
        let changed_endpoint = ContainerRegistryEndpoint {
            login_server: "fixture-cd34.azurecr.io".parse()?,
            ..endpoint.clone()
        };
        let original = AzureContainerRegistryRepositoryListRequest {
            endpoint: Cow::Borrowed(&endpoint),
            authentication,
        };
        let changed = AzureContainerRegistryRepositoryListRequest {
            endpoint: Cow::Borrowed(&changed_endpoint),
            authentication,
        };
        assert_ne!(original.cache_key()?, changed.cache_key()?);
        assert_eq!(
            original.cache_keys_to_invalidate()?,
            vec![original.cache_key()?]
        );

        let tags = AzureContainerRegistryRepositoryTagsRequest {
            endpoint: Cow::Borrowed(&endpoint),
            repository_name: Cow::Owned("team/image".parse()?),
            authentication,
        };
        let changed_tags = AzureContainerRegistryRepositoryTagsRequest {
            endpoint: Cow::Borrowed(&changed_endpoint),
            repository_name: Cow::Borrowed(&tags.repository_name),
            authentication,
        };
        let other_repository = AzureContainerRegistryRepositoryTagsRequest {
            endpoint: Cow::Borrowed(&endpoint),
            repository_name: Cow::Owned("team%2Fimage".parse()?),
            authentication,
        };
        assert_ne!(tags.cache_key()?, changed_tags.cache_key()?);
        assert_ne!(tags.cache_key()?, other_repository.cache_key()?);
        assert_eq!(tags.cache_keys_to_invalidate()?, vec![tags.cache_key()?]);
        assert!(
            tags.cache_key()?
                .scope
                .starts_with(&original.cache_key()?.scope)
        );
        Ok(())
    }
}
