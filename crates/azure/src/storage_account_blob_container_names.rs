use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::StorageAccountBlobContainerListResponse;
use cloud_terrastodon_azure_types::StorageAccountBlobContainerName;
use cloud_terrastodon_azure_types::StorageAccountId;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::HasCacheKey;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use eyre::eyre;
use http::Method;
use std::borrow::Cow;
use std::collections::HashSet;
use std::future::Future;
use std::future::IntoFuture;
use std::path::PathBuf;
use std::pin::Pin;
use url::Url;

const CONTAINERS_API_VERSION: &str = "2025-06-01";

/// List an account's container names through Azure Resource Manager.
///
/// Uses [Blob Containers - List, 2025-06-01](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01).
/// This is management-plane inventory: it does not demonstrate permission to
/// read or write blobs through the storage data plane. Soft-deleted containers
/// are not requested. Invalidating this request refreshes all its cached pages.
///
/// Although the API's introduction says continuation is unsupported, its response
/// schema and official `x-ms-pageable` contract define `nextLink`; this request
/// follows it when present.
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct StorageAccountBlobContainerNamesListRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
    pub storage_account_id: Cow<'a, StorageAccountId>,
}

/// List management-plane container names using the explicitly selected tenant.
///
/// See [Blob Containers - List, 2025-06-01](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01).
pub fn fetch_storage_account_blob_container_names<'a>(
    storage_account_id: &'a StorageAccountId,
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> StorageAccountBlobContainerNamesListRequest<'a> {
    StorageAccountBlobContainerNamesListRequest {
        cancellation: cancellation.clone(),
        auth_context: Cow::Borrowed(auth_context),
        storage_account_id: Cow::Borrowed(storage_account_id),
    }
}

impl<'a> Arbitrary<'a> for StorageAccountBlobContainerNamesListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(Arbitrary::arbitrary(u)?),
            storage_account_id: Cow::Owned(StorageAccountId::arbitrary(u)?),
        })
    }
}

impl HasCacheKey for StorageAccountBlobContainerNamesListRequest<'_> {
    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            PathBuf::from("az/arm/tenant")
                .join(self.auth_context.tenant_id.to_string())
                .join("auth-source")
                .join(
                    self.auth_context
                        .auth_context
                        .source()
                        .map(|source| source.to_string())
                        .unwrap_or_else(|| "placeholder".to_owned()),
                )
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
                .join("blob-containers/list"),
        )
    }
}

impl<'a> CacheInvalidatableIntoFuture for StorageAccountBlobContainerNamesListRequest<'a> {
    type WithInvalidation = <Self as IntoFuture>::IntoFuture;

    fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
        Box::pin(async move {
            if invalidate_cache {
                self.cancellation
                    .deferred(
                        "finishing filesystem and memory cache invalidation",
                        |_invalidation| async { self.invalidate().await },
                    )
                    .await?;
            }
            self.into_future().await
        })
    }
}

impl<'a> IntoFuture for StorageAccountBlobContainerNamesListRequest<'a> {
    type Output = Result<HashSet<StorageAccountBlobContainerName>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Keep REST page entries distinct from any future aggregate entry.
            let page_cache = self
                .cache_key()
                .join("rest/api-version")
                .join(CONTAINERS_API_VERSION)
                .join("pages");
            let mut url = Url::parse("https://management.azure.com")
                .wrap_err("constructing the ARM container collection endpoint")?;
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
                    "blobServices",
                    "default",
                    "containers",
                ]);
            url.query_pairs_mut()
                .append_pair("api-version", CONTAINERS_API_VERSION);
            let collection_path = url.path().to_owned();
            let mut next = Some(url);
            let mut visited = HashSet::new();
            let mut names = HashSet::new();
            while let Some(url) = next {
                self.cancellation.bail_if_cancelled()?;
                Self::validate_page_url(&url, &collection_path)?;
                ensure!(
                    visited.insert(url.clone()),
                    "container inventory pagination repeated a previously requested URL"
                );
                let page_key =
                    page_cache.join(blake3::hash(url.as_str().as_bytes()).to_hex().as_str());
                let page = RestRequest::from_method_and_url(Method::GET, url, &self.cancellation)?
                    .azure_auth_context(&self.auth_context)
                    .cache(page_key)
                    .receive::<StorageAccountBlobContainerListResponse>()
                    .await?;
                self.cancellation.bail_if_cancelled()?;
                for container in page.value {
                    self.cancellation.bail_if_cancelled()?;
                    names.insert(container.name);
                }
                next = page.next_link;
            }
            Ok(names)
        })
    }
}

impl StorageAccountBlobContainerNamesListRequest<'_> {
    fn validate_page_url(url: &Url, collection_path: &str) -> Result<()> {
        // A library credential-boundary constraint: continuation targets only
        // this account's collection on the supported public-cloud ARM host.
        ensure!(
            url.scheme() == "https"
                && url.host_str() == Some("management.azure.com")
                && url.port_or_known_default() == Some(443)
                && url.username().is_empty()
                && url.password().is_none()
                && url.fragment().is_none()
                && url.path().eq_ignore_ascii_case(collection_path),
            "container inventory continuation URL leaves the selected ARM container collection"
        );
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(StorageAccountBlobContainerNamesListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    StorageAccountBlobContainerNamesListRequest<'static>
);
cloud_terrastodon_registry_execution::register_into_future!(
    StorageAccountBlobContainerNamesListRequest<'static> => HashSet<StorageAccountBlobContainerName>,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::StorageAccountBlobContainerNamesListRequest;
    use super::fetch_storage_account_blob_container_names;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth_types::AuthSource;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_command::HasCacheKey;
    use eyre::Result;

    #[test]
    fn continuation_cannot_change_account_or_credential_destination() -> Result<()> {
        let path = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/synthetic/providers/Microsoft.Storage/storageAccounts/syntheticaccount/blobServices/default/containers";
        let url =
            format!("https://management.azure.com{path}?api-version=2025-06-01&%24skipToken=next")
                .parse()?;
        StorageAccountBlobContainerNamesListRequest::validate_page_url(&url, path)?;
        for url in [
            format!("http://management.azure.com{path}"),
            format!("https://other.example{path}"),
            format!("https://management.azure.com:444{path}"),
            format!("https://user@management.azure.com{path}"),
            format!("https://management.azure.com{path}#fragment"),
            format!(
                "https://management.azure.com{}",
                path.replace("syntheticaccount", "otheraccount")
            ),
        ] {
            assert!(
                StorageAccountBlobContainerNamesListRequest::validate_page_url(&url.parse()?, path)
                    .is_err()
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn cache_identity_separates_tenant_source_and_account() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let first_tenant = "11111111-1111-1111-1111-111111111111".parse()?;
                let second_tenant = "22222222-2222-2222-2222-222222222222".parse()?;
                let first_cli = AuthContext::explicit(AuthSource::AzureCli).bind_to_azure_tenant(first_tenant)?;
                let second_cli = AuthContext::explicit(AuthSource::AzureCli).bind_to_azure_tenant(second_tenant)?;
                let first_browser = AuthContext::explicit(AuthSource::Browser).bind_to_azure_tenant(first_tenant)?;
                let account = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/synthetic/providers/Microsoft.Storage/storageAccounts/syntheticaccount".parse()?;
                let other = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/synthetic/providers/Microsoft.Storage/storageAccounts/otheraccount".parse()?;
                let key = fetch_storage_account_blob_container_names(&account, &first_cli, &cancellation).cache_key();
                for (account, context) in [(&account, &second_cli), (&account, &first_browser), (&other, &first_cli)] {
                    let distinct = fetch_storage_account_blob_container_names(account, context, &cancellation).cache_key();
                    assert_ne!(key.path, distinct.path);
                    assert!(!key.path.starts_with(&distinct.path));
                    assert!(!distinct.path.starts_with(&key.path));
                }
                Ok(())
            })
            .await
    }
}
