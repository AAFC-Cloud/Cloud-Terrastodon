use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ComputePublisherName;
use cloud_terrastodon_azure_types::ComputePublisherVmImageOfferName;
use cloud_terrastodon_azure_types::ComputePublisherVmImageOfferSkuName;
use cloud_terrastodon_azure_types::ComputePublisherVmImageOfferSkuVersionId;
use cloud_terrastodon_azure_types::ComputePublisherVmImageOfferSkuVersionReference;
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

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct ComputePublisherImageOfferSkuVersionListRequest {
    pub auth_source: AzureBearerAuthSource,
    pub tenant_id: AzureTenantId,

    pub subscription_id: SubscriptionId,
    pub location: AzureLocationName,
    pub publisher_name: ComputePublisherName,
    pub offer_name: ComputePublisherVmImageOfferName,
    pub sku_name: ComputePublisherVmImageOfferSkuName,
}

#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "The image hierarchy and authentication selection remain explicit operation inputs"
)]
pub fn fetch_compute_publisher_image_offer_sku_versions<'exec>(
    subscription_id: SubscriptionId,
    location: AzureLocationName,
    publisher_name: ComputePublisherName,
    offer_name: ComputePublisherVmImageOfferName,
    sku_name: ComputePublisherVmImageOfferSkuName,
    auth_source: AzureBearerAuthSource,
    tenant_id: AzureTenantId,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, ComputePublisherImageOfferSkuVersionListRequest> {
    executor.execute(ComputePublisherImageOfferSkuVersionListRequest {
        auth_source,
        tenant_id,

        subscription_id,
        location,
        publisher_name,
        offer_name,
        sku_name,
    })
}

impl InvalidatesOwnCacheKey for ComputePublisherImageOfferSkuVersionListRequest {}

impl HasCacheKey for ComputePublisherImageOfferSkuVersionListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "vm",
            "list-publishers-offers-sku-versions",
            self.subscription_id.to_string().as_str(),
            self.location.to_string().as_str(),
            self.publisher_name.to_string().as_str(),
            self.offer_name.to_string().as_str(),
            self.sku_name.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for ComputePublisherImageOfferSkuVersionListRequest {
    type Error = eyre::Report;
    type Output = Vec<ComputePublisherVmImageOfferSkuVersionId>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = format!(
                "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/publishers/{publisher_name}/artifacttypes/vmimage/offers/{offer_name}/skus/{sku_name}/versions?$expand=properties%2FimageDeprecationStatus&api-version=2024-07-01",
                subscription_id = self.subscription_id,
                location = self.location,
                publisher_name = self.publisher_name,
                offer_name = self.offer_name,
                sku_name = self.sku_name
            );
            let rtn = RestRequest::new(http::Method::GET, &url)?
                .authentication(AuthenticationSelection::AzureBearer {
                    auth_source: self.auth_source,
                    tenant_id: self.tenant_id,
                })
                .cache(self.cache_key()?)
                .receive_cached::<Vec<ComputePublisherVmImageOfferSkuVersionReference>>(
                    &executor,
                    CacheOperationOwner::of::<ComputePublisherImageOfferSkuVersionListRequest>(),
                )
                .await?
                .iter()
                .map(|row| row.id.clone())
                .collect();
            Ok(rtn)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::fetch_all_subscriptions;
    use crate::fetch_compute_publisher_image_offer_sku_versions;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::AzureLocationName;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let subscription_id = fetch_all_subscriptions(
                    &AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?,
                    &executor,
                )
                .await?
                .first()
                .unwrap()
                .id;
                let publisher = "center-for-internet-security-inc".parse()?;
                let offer = "cis-windows-server-2016-v1-0-0-l2".parse()?;
                let sku = "cis-ws2016-l2".parse()?;
                let sku_versions = fetch_compute_publisher_image_offer_sku_versions(
                    subscription_id,
                    AzureLocationName::CanadaCentral,
                    publisher,
                    offer,
                    sku,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    tenant_id,
                    &executor,
                )
                .await?;
                assert!(!sku_versions.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ComputePublisherImageOfferSkuVersionListRequest);
cloud_terrastodon_registry::register_arbitrary!(ComputePublisherImageOfferSkuVersionListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(ComputePublisherImageOfferSkuVersionListRequest => Vec<ComputePublisherVmImageOfferSkuVersionId>, effects = [Read]);
