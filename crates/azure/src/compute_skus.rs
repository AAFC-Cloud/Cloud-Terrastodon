use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ComputeSku;
use cloud_terrastodon_azure_types::ComputeSkuListResponse;
use cloud_terrastodon_azure_types::SubscriptionId;
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

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct ComputeSkuListRequest {
    pub auth_source: AzureBearerAuthSource,
    pub tenant_id: AzureTenantId,

    pub subscription_id: SubscriptionId,
}

#[track_caller]
pub fn fetch_all_compute_skus<'exec>(
    subscription_id: SubscriptionId,
    auth_source: AzureBearerAuthSource,
    tenant_id: AzureTenantId,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, ComputeSkuListRequest> {
    executor.execute(ComputeSkuListRequest {
        auth_source,
        tenant_id,

        subscription_id,
    })
}

impl InvalidatesOwnCacheKey for ComputeSkuListRequest {}

impl HasCacheKey for ComputeSkuListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "vm",
            "list-skus",
            self.subscription_id.to_string().as_ref(),
        ]))
    }
}

impl ExecutableRequest for ComputeSkuListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<ComputeSku>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                "Fetching all VM SKUs for subscription {}",
                self.subscription_id
            );
            let url = format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/skus?api-version=2019-04-01",
                self.subscription_id
            );
            let rtn = RestRequest::new(http::Method::GET, &url)?
                .authentication(AuthenticationSelection::AzureBearer {
                    auth_source: self.auth_source,
                    tenant_id: self.tenant_id,
                })
                .cache(self.cache_key()?)
                .receive_cached::<ComputeSkuListResponse>(
                    &executor,
                    CacheOperationOwner::of::<ComputeSkuListRequest>(),
                )
                .await?
                .value
                .clone();
            debug!(
                "Found {} VM SKUs for subscription {}",
                rtn.len(),
                self.subscription_id
            );
            Ok(rtn)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::fetch_all_compute_skus;
    use crate::fetch_all_subscriptions;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::AzureLocationName;
    use cloud_terrastodon_azure_types::ComputeSkuResourceType;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    #[ignore] // this endpoint takes forever
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let subs = fetch_all_subscriptions(
                    &AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?,
                    &executor,
                )
                .await?;
                let sub = subs.first().unwrap();
                let vm_skus = fetch_all_compute_skus(
                    sub.id,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    tenant_id,
                    &executor,
                )
                .await?;
                let canada_vm_skus = vm_skus
                    .iter()
                    .filter(|s| s.locations.iter().any(AzureLocationName::is_canada))
                    .collect::<Vec<_>>();
                let vm_skus_in_canada = canada_vm_skus
                    .into_iter()
                    .filter(|sku| sku.resource_type == ComputeSkuResourceType::VirtualMachines)
                    .count();
                assert!(vm_skus_in_canada > 0);
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ComputeSkuListRequest);
cloud_terrastodon_registry::register_arbitrary!(ComputeSkuListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(ComputeSkuListRequest => Vec<Arc<ComputeSku>>, effects = [Read]);
