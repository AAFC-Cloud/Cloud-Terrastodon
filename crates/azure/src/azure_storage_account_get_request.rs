use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureStorageAccount;
use cloud_terrastodon_azure_types::AzureStorageAccountId;
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

/// Read one storage account directly through Azure Resource Manager.
///
/// Uses [Storage Accounts - Get Properties, 2025-06-01](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01).
/// This reads account configuration, not account keys or stored data. It does
/// not request optional `$expand` values. Invalidating it refreshes this account's
/// get-properties result without clearing inventories or blob-container reads.
#[must_use = "Execute this request with an executor"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureStorageAccountGetRequest<'a> {
    pub authentication: AzureBearerAuthenticationSelection,
    pub storage_account_id: Cow<'a, AzureStorageAccountId>,
}

impl<'a> Arbitrary<'a> for AzureStorageAccountGetRequest<'static> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: Arbitrary::arbitrary(u)?,
            storage_account_id: Cow::Owned(Arbitrary::arbitrary(u)?),
        })
    }
}

impl HasCacheKey for AzureStorageAccountGetRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(
            PathBuf::from("az/arm/tenant")
                .join(self.authentication.tenant_id.to_string())
                .join("auth-source")
                .join(self.authentication.auth_source.to_string())
                .join("subscriptions")
                .join(
                    self.storage_account_id
                        .resource_group_id
                        .subscription_id
                        .to_string(),
                )
                .join("resource-groups")
                .join(sanitize_windows_path_component(
                    &self
                        .storage_account_id
                        .resource_group_id
                        .resource_group_name
                        .to_string()
                        .to_lowercase(),
                ))
                .join("storage-accounts")
                .join(sanitize_windows_path_component(
                    &self.storage_account_id.storage_account_name.to_string(),
                ))
                .join("get"),
        )
    }
}

impl InvalidatesOwnCacheKey for AzureStorageAccountGetRequest<'_> {}

impl ExecutableRequest for AzureStorageAccountGetRequest<'_> {
    type Error = eyre::Report;
    type Output = Arc<AzureStorageAccount>;

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
                .wrap_err("constructing the ARM storage account endpoint")?;
            url.path_segments_mut()
                .map_err(|_| eyre!("the ARM endpoint cannot contain path segments"))?
                .extend([
                    "subscriptions",
                    &self
                        .storage_account_id
                        .resource_group_id
                        .subscription_id
                        .to_string(),
                    "resourceGroups",
                    &self
                        .storage_account_id
                        .resource_group_id
                        .resource_group_name
                        .to_string(),
                    "providers",
                    "Microsoft.Storage",
                    "storageAccounts",
                    &self.storage_account_id.storage_account_name.to_string(),
                ]);
            url.query_pairs_mut()
                .append_pair("api-version", "2025-06-01");
            RestRequest::from_method_and_url(Method::GET, url)?
                .authentication(self.authentication.into())
                .cache(cache_key)
                .receive_cached(
                    &executor,
                    CacheOperationOwner::of::<AzureStorageAccountGetRequest<'static>>(),
                )
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountGetRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureStorageAccountGetRequest<'static> => Arc<AzureStorageAccount>,
    effects = [Read]
);
cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureStorageAccountGetRequest<'static>,
    name = "storage-account-get",
    cached
);

#[cfg(test)]
mod tests {
    use super::AzureStorageAccountGetRequest;
    use crate::AzureStorageAccountBlobContainerListRequest;
    use crate::AzureStorageAccountListRequest;
    use crate::test_authentication::test_authentication;
    use cloud_terrastodon_azure_types::AzureStorageAccountId;
    use cloud_terrastodon_cache::HasCacheKey;
    use std::borrow::Cow;

    #[test]
    fn account_refresh_is_confined_to_its_get_without_invalidating_child_inventories()
    -> eyre::Result<()> {
        let id: AzureStorageAccountId = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/synthetic/providers/Microsoft.Storage/storageAccounts/syntheticaccount".parse()?;
        let get = AzureStorageAccountGetRequest {
            authentication: test_authentication(),
            storage_account_id: Cow::Borrowed(&id),
        }
        .cache_key()?;
        let containers = AzureStorageAccountBlobContainerListRequest {
            authentication: test_authentication(),
            storage_account_id: Cow::Borrowed(&id),
        }
        .cache_key()?;
        let accounts = AzureStorageAccountListRequest {
            authentication: test_authentication(),
        }
        .cache_key()?;
        let other = AzureStorageAccountGetRequest {
            authentication: test_authentication(),
            storage_account_id: Cow::Owned(AzureStorageAccountId {
                storage_account_name: "otheraccount".parse()?,
                ..id
            }),
        }
        .cache_key()?;
        for independent in [containers, accounts, other] {
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
