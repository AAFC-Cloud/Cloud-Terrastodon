use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureStorageAccount;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use indoc::indoc;
use std::path::PathBuf;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureStorageAccountListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureStorageAccountListRequest {}

impl HasCacheKey for AzureStorageAccountListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "storage_accounts",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureStorageAccountListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureStorageAccount>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            ResourceGraphHelper::new(
                indoc! {r#"
                Resources
                | where type == "microsoft.storage/storageaccounts"
                | project id,name,kind,location,sku,properties,tags
            "#},
                Some(self.cache_key()?),
                self.authentication,
            )
            .collect_all::<AzureStorageAccount>(&executor)
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureStorageAccountListRequest => Vec<Arc<AzureStorageAccount>>, effects = [Read]);
cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureStorageAccountListRequest,
    name = "storage-account-list",
    cached
);
