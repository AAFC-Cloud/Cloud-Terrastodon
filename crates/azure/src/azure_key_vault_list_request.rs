use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::KeyVault;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet, Arbitrary)]
#[facet(traits(Clone))]
pub struct AzureKeyVaultListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureKeyVaultListRequest {}

impl HasCacheKey for AzureKeyVaultListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "key_vaults",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureKeyVaultListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<KeyVault>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let mut query = ResourceGraphHelper::new(
                r#"
resources
| where type =~ "microsoft.keyvault/vaults"
| project id,name,location,properties,tags
        "#,
                Some(self.cache_key()?),
                self.authentication,
            );
            query.collect_all(&executor).await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureKeyVaultListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureKeyVaultListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureKeyVaultListRequest => Vec<Arc<KeyVault>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureKeyVaultListRequest,
    name = "key-vault-list",
    cached
);
