use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureStorageAccountBlobContainerListResponse;
use cloud_terrastodon_azure_types::AzureStorageAccountBlobContainerName;
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
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use eyre::eyre;
use facet::Facet;
use http::Method;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
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
#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureStorageAccountBlobContainerNamesListRequest<'a> {
    pub authentication: AzureBearerAuthenticationSelection,
    pub storage_account_id: Cow<'a, AzureStorageAccountId>,
}

impl<'a> Arbitrary<'a> for AzureStorageAccountBlobContainerNamesListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: Arbitrary::arbitrary(u)?,
            storage_account_id: Cow::Owned(AzureStorageAccountId::arbitrary(u)?),
        })
    }
}

impl HasCacheKey for AzureStorageAccountBlobContainerNamesListRequest<'_> {
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
                .join("blob-containers/list"),
        )
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureStorageAccountBlobContainerNamesListRequest<'a> {}

impl<'a> ExecutableRequest for AzureStorageAccountBlobContainerNamesListRequest<'a> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = HashSet<AzureStorageAccountBlobContainerName>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            // Keep REST page entries distinct from any future aggregate entry.
            let page_cache = self
                .cache_key()?
                .join("rest/api-version")?
                .join(CONTAINERS_API_VERSION)?
                .join("pages")?;
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
                executor.cancellation().bail_if_cancelled()?;
                Self::validate_page_url(&url, &collection_path)?;
                ensure!(
                    visited.insert(url.clone()),
                    "container inventory pagination repeated a previously requested URL"
                );
                let page_key =
                    page_cache.join(blake3::hash(url.as_str().as_bytes()).to_hex().as_str())?;
                let page = RestRequest::from_method_and_url(Method::GET, url)?
                    .authentication(self.authentication.into())
                    .cache(page_key)
                    .receive_cached::<AzureStorageAccountBlobContainerListResponse>(
                        &executor,
                        CacheOperationOwner::of::<
                            AzureStorageAccountBlobContainerNamesListRequest<'static>,
                        >(),
                    )
                    .await?;
                executor.cancellation().bail_if_cancelled()?;
                for container in &page.value {
                    executor.cancellation().bail_if_cancelled()?;
                    names.insert(container.name.clone());
                }
                next = page.next_link.clone();
            }
            Ok(names)
        })
    }
}

impl AzureStorageAccountBlobContainerNamesListRequest<'_> {
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

cloud_terrastodon_registry::register_thing!(
    AzureStorageAccountBlobContainerNamesListRequest<'static>
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureStorageAccountBlobContainerNamesListRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureStorageAccountBlobContainerNamesListRequest<'static> => HashSet<AzureStorageAccountBlobContainerName>,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::AzureStorageAccountBlobContainerNamesListRequest;
    use crate::test_authentication::test_authentication;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
    use cloud_terrastodon_cache::HasCacheKey;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::Result;
    use std::borrow::Cow;

    #[test]
    fn continuation_cannot_change_account_or_credential_destination() -> Result<()> {
        let path = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/synthetic/providers/Microsoft.Storage/storageAccounts/syntheticaccount/blobServices/default/containers";
        let url =
            format!("https://management.azure.com{path}?api-version=2025-06-01&%24skipToken=next")
                .parse()?;
        AzureStorageAccountBlobContainerNamesListRequest::validate_page_url(&url, path)?;
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
                AzureStorageAccountBlobContainerNamesListRequest::validate_page_url(
                    &url.parse()?,
                    path
                )
                .is_err()
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn cache_identity_separates_tenant_source_and_account() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|_cancellation| async move {
                let first_tenant = "11111111-1111-1111-1111-111111111111".parse()?;
                let second_tenant = "22222222-2222-2222-2222-222222222222".parse()?;
                let first_cli = AzureBearerAuthenticationSelection { tenant_id: first_tenant, ..test_authentication() };
                let second_cli = AzureBearerAuthenticationSelection { tenant_id: second_tenant, ..test_authentication() };
                let first_browser = AzureBearerAuthenticationSelection { tenant_id: first_tenant, auth_source: AzureBearerAuthSource::Browser, ..test_authentication() };
                let account = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/synthetic/providers/Microsoft.Storage/storageAccounts/syntheticaccount".parse()?;
                let other = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/synthetic/providers/Microsoft.Storage/storageAccounts/otheraccount".parse()?;
                let key = AzureStorageAccountBlobContainerNamesListRequest { authentication: first_cli, storage_account_id: Cow::Borrowed(&account) }.cache_key()?;
                for (account, context) in [(&account, &second_cli), (&account, &first_browser), (&other, &first_cli)] {
                    let distinct = AzureStorageAccountBlobContainerNamesListRequest { authentication: *context, storage_account_id: Cow::Borrowed(account) }.cache_key()?;
                    assert_ne!(key.scope, distinct.scope);
                    assert!(!key.scope.starts_with(&distinct.scope));
                    assert!(!distinct.scope.starts_with(&key.scope));
                }
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureStorageAccountBlobContainerNamesListRequest<'static>,
    name = "storage-account-blob-container-names-list",
    cached
);
