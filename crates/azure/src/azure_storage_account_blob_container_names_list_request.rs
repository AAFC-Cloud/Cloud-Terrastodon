use crate::AzureStorageAccountBlobContainerListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureStorageAccountBlobContainerName;
use cloud_terrastodon_azure_types::AzureStorageAccountId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::borrow::Cow;
use std::collections::HashSet;

/// List an account's container names through Azure Resource Manager.
///
/// Projects the names from [`AzureStorageAccountBlobContainerListRequest`],
/// sharing its decoded pages and invalidation scope. This management-plane
/// inventory does not demonstrate permission to access blob data.
/// [Blob Containers - List, 2025-06-01](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01).
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
        AzureStorageAccountBlobContainerListRequest {
            authentication: self.authentication,
            storage_account_id: Cow::Borrowed(&self.storage_account_id),
        }
        .cache_key()
    }
}

impl InvalidatesOwnCacheKey for AzureStorageAccountBlobContainerNamesListRequest<'_> {}

impl ExecutableRequest for AzureStorageAccountBlobContainerNamesListRequest<'_> {
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
            let containers = AzureStorageAccountBlobContainerListRequest {
                authentication: self.authentication,
                storage_account_id: self.storage_account_id,
            }
            .execute(&executor)
            .await?;
            let mut names = HashSet::with_capacity(containers.len());
            for container in containers {
                executor.cancellation().bail_if_cancelled()?;
                names.insert(container.name.clone());
            }
            Ok(names)
        })
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
cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureStorageAccountBlobContainerNamesListRequest<'static>,
    name = "storage-account-blob-container-names-list",
    cached
);
