use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureProviderOperationsListResponse;
use cloud_terrastodon_azure_types::AzureProviderOperationsMetadata;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

const PROVIDER_OPERATIONS_API_VERSION: &str = "2022-04-01";

#[must_use = "This is a future request, you must .execute it"]
#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct RoleOperationListRequest {
    pub auth_source: AzureBearerAuthSource,

    pub tenant_id: AzureTenantId,
}

#[track_caller]
pub fn fetch_all_role_operation_metadata<'exec>(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, RoleOperationListRequest> {
    executor.execute(RoleOperationListRequest {
        auth_source,

        tenant_id,
    })
}

impl InvalidatesOwnCacheKey for RoleOperationListRequest {}

impl HasCacheKey for RoleOperationListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "providerOperations",
            self.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for RoleOperationListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureProviderOperationsMetadata>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let tenant_id = self.tenant_id.to_string();
            let mut next_url = Some(format!(
                "https://management.azure.com/providers/Microsoft.Authorization/providerOperations?$expand=resourceTypes&api-version={PROVIDER_OPERATIONS_API_VERSION}"
            ));
            let mut page_index = 0usize;
            let mut operations = Vec::new();

            while let Some(url) = next_url.take() {
                debug!(page_index, %url, %tenant_id, "Fetching Azure provider operations metadata");
                let response: Arc<AzureProviderOperationsListResponse> =
                    RestRequest::new(http::Method::GET, &url)?
                        .authentication(AuthenticationSelection::AzureBearer {
                            auth_source: self.auth_source,
                            tenant_id: self.tenant_id,
                        })
                        .cache(CacheKey {
                            scope: self.cache_key()?.scope.join(page_index.to_string())?,
                            valid_for: self.cache_key()?.valid_for,
                        })
                        .receive_cached(
                            &executor,
                            CacheOperationOwner::of::<RoleOperationListRequest>(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::Result;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let results = fetch_all_role_operation_metadata(
                    get_test_tenant_id(&cancellation).await?,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    &executor,
                )
                .await?;
                assert!(!results.is_empty());
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn includes_microsoft_authorization_provider() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let results = fetch_all_role_operation_metadata(
                    get_test_tenant_id(&cancellation).await?,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    &executor,
                )
                .await?;
                assert!(
                    results
                        .iter()
                        .any(|provider| provider.name == "Microsoft.Authorization")
                );
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(RoleOperationListRequest);
cloud_terrastodon_registry::register_arbitrary!(RoleOperationListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(RoleOperationListRequest => Vec<Arc<AzureProviderOperationsMetadata>>, effects = [Read]);
