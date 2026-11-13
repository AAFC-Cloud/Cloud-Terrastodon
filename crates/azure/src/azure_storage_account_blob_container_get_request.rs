use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureStorageAccountBlobContainer;
use cloud_terrastodon_azure_types::AzureStorageAccountBlobContainerId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::WrapErr;
use eyre::eyre;
use facet::Facet;
use http::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

/// Read one container's properties directly through Azure Resource Manager.
///
/// Uses [Blob Containers - Get, 2025-06-01](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/get?view=rest-storagerp-2025-06-01).
/// This management-plane request neither lists the account's other containers
/// nor reads blobs. Invalidating it refreshes only this container's cached get.
#[must_use = "Execute this request with an executor"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureStorageAccountBlobContainerGetRequest<'a> {
    pub authentication: AzureBearerAuthenticationSelection,
    pub container_id: Cow<'a, AzureStorageAccountBlobContainerId>,
}

impl<'a> Arbitrary<'a> for AzureStorageAccountBlobContainerGetRequest<'static> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: Arbitrary::arbitrary(u)?,
            container_id: Cow::Owned(Arbitrary::arbitrary(u)?),
        })
    }
}

impl HasCacheKey for AzureStorageAccountBlobContainerGetRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(
            PathBuf::from("az/arm/tenant")
                .join(self.authentication.tenant_id.to_string())
                .join("auth-source")
                .join(self.authentication.auth_source.to_string())
                .join("subscriptions")
                .join(
                    self.container_id
                        .storage_account_id
                        .resource_group_id
                        .subscription_id
                        .to_string(),
                )
                .join("resource-groups")
                .join(sanitize_windows_path_component(
                    &self
                        .container_id
                        .storage_account_id
                        .resource_group_id
                        .resource_group_name
                        .to_string()
                        .to_lowercase(),
                ))
                .join("storage-accounts")
                .join(sanitize_windows_path_component(
                    &self
                        .container_id
                        .storage_account_id
                        .storage_account_name
                        .to_string(),
                ))
                .join("blob-containers/get")
                .join(sanitize_windows_path_component(
                    &self.container_id.container_name.to_string(),
                )),
        )
    }
}

impl InvalidatesOwnCacheKey for AzureStorageAccountBlobContainerGetRequest<'_> {}

impl ExecutableRequest for AzureStorageAccountBlobContainerGetRequest<'_> {
    type Error = eyre::Report;
    type Output = Arc<AzureStorageAccountBlobContainer>;

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
            let cache_key = self.cache_key()?.join("rest/api-version/2025-06-01")?;
            let mut url = Url::parse("https://management.azure.com")
                .wrap_err("constructing the ARM blob container endpoint")?;
            url.path_segments_mut()
                .map_err(|_| eyre!("the ARM endpoint cannot contain path segments"))?
                .extend([
                    "subscriptions",
                    &self
                        .container_id
                        .storage_account_id
                        .resource_group_id
                        .subscription_id
                        .to_string(),
                    "resourceGroups",
                    &self
                        .container_id
                        .storage_account_id
                        .resource_group_id
                        .resource_group_name
                        .to_string(),
                    "providers",
                    "Microsoft.Storage",
                    "storageAccounts",
                    &self
                        .container_id
                        .storage_account_id
                        .storage_account_name
                        .to_string(),
                    "blobServices",
                    "default",
                    "containers",
                    &self.container_id.container_name.to_string(),
                ]);
            url.query_pairs_mut()
                .append_pair("api-version", "2025-06-01");
            RestRequest::from_method_and_url(Method::GET, url)?
                .authentication(self.authentication.into())
                .cache(cache_key)
                .receive_cached(
                    &executor,
                    CacheOperationOwner::of::<AzureStorageAccountBlobContainerGetRequest<'static>>(
                    ),
                )
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobContainerGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    AzureStorageAccountBlobContainerGetRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureStorageAccountBlobContainerGetRequest<'static> => Arc<AzureStorageAccountBlobContainer>,
    effects = [Read]
);
cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureStorageAccountBlobContainerGetRequest<'static>,
    name = "storage-account-blob-container-get",
    cached
);

#[cfg(test)]
mod tests {
    use super::AzureStorageAccountBlobContainerGetRequest;
    use crate::AzureStorageAccountBlobContainerListRequest;
    use crate::test_authentication::test_authentication;
    use cloud_terrastodon_azure_types::AzureStorageAccountBlobContainerId;
    use cloud_terrastodon_cache::HasCacheKey;
    use std::borrow::Cow;

    #[test]
    fn container_refresh_does_not_invalidate_other_containers_or_the_list() -> eyre::Result<()> {
        let id: AzureStorageAccountBlobContainerId = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/synthetic/providers/Microsoft.Storage/storageAccounts/syntheticaccount/blobServices/default/containers/first".parse()?;
        let get = AzureStorageAccountBlobContainerGetRequest {
            authentication: test_authentication(),
            container_id: Cow::Borrowed(&id),
        }
        .cache_key()?;
        let list = AzureStorageAccountBlobContainerListRequest {
            authentication: test_authentication(),
            storage_account_id: Cow::Borrowed(&id.storage_account_id),
        }
        .cache_key()?;
        let other = AzureStorageAccountBlobContainerGetRequest {
            authentication: test_authentication(),
            container_id: Cow::Owned(AzureStorageAccountBlobContainerId {
                container_name: "second".parse()?,
                ..id
            }),
        }
        .cache_key()?;
        for independent in [list, other] {
            assert!(!independent.scope.starts_with(&get.scope));
            assert!(!get.scope.starts_with(&independent.scope));
        }
        assert!(
            get.join("rest/api-version/2025-06-01")?
                .scope
                .starts_with(&get.scope)
        );
        Ok(())
    }
}
