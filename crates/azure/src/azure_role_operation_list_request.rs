use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureProviderOperationsListResponse;
use cloud_terrastodon_azure_types::AzureProviderOperationsMetadata;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

const PROVIDER_OPERATIONS_API_VERSION: &str = "2022-04-01";

#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct AzureRoleOperationListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureRoleOperationListRequest {}

impl HasCacheKey for AzureRoleOperationListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "providerOperations",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureRoleOperationListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureProviderOperationsMetadata>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let tenant_id = self.authentication.tenant_id.to_string();
            let mut next_url = Some(format!(
                "https://management.azure.com/providers/Microsoft.Authorization/providerOperations?$expand=resourceTypes&api-version={PROVIDER_OPERATIONS_API_VERSION}"
            ));
            let mut page_index = 0usize;
            let mut operations = Vec::new();

            while let Some(url) = next_url.take() {
                debug!(page_index, %url, %tenant_id, "Fetching Azure provider operations metadata");
                let response: Arc<AzureProviderOperationsListResponse> =
                    RestRequest::new(http::Method::GET, &url)?
                        .authentication(self.authentication.into())
                        .cache(CacheKey {
                            scope: self.cache_key()?.scope.join(page_index.to_string())?,
                            valid_for: self.cache_key()?.valid_for,
                        })
                        .receive_cached(
                            &executor,
                            CacheOperationOwner::of::<AzureRoleOperationListRequest>(),
                        )
                        .await?;
                operations.extend(response.value.iter().cloned());
                next_url = response.next_link.clone();
                page_index += 1;
            }

            debug!(count = operations.len(), %tenant_id, "Fetched Azure provider operations metadata");
            Ok(operations)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureRoleOperationListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureRoleOperationListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureRoleOperationListRequest => Vec<Arc<AzureProviderOperationsMetadata>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureRoleOperationListRequest,
    name = "role-operation-list",
    cached
);
