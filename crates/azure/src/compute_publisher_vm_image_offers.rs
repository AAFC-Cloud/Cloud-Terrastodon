use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ComputePublisherName;
use cloud_terrastodon_azure_types::ComputePublisherVmImageOfferId;
use cloud_terrastodon_azure_types::ComputePublisherVmImageOfferReference;
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
pub struct ComputePublisherImageOfferListRequest {
    pub auth_source: AzureBearerAuthSource,
    pub tenant_id: AzureTenantId,

    pub subscription_id: SubscriptionId,
    pub location: AzureLocationName,
    pub publisher_name: ComputePublisherName,
}

#[track_caller]
pub fn fetch_compute_publisher_image_offers<'exec>(
    subscription_id: SubscriptionId,
    location: AzureLocationName,
    publisher_name: ComputePublisherName,
    auth_source: AzureBearerAuthSource,
    tenant_id: AzureTenantId,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, ComputePublisherImageOfferListRequest> {
    executor.execute(ComputePublisherImageOfferListRequest {
        auth_source,
        tenant_id,

        subscription_id,
        location,
        publisher_name,
    })
}

impl InvalidatesOwnCacheKey for ComputePublisherImageOfferListRequest {}

impl HasCacheKey for ComputePublisherImageOfferListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "vm",
            "list-publishers-offers",
            self.subscription_id.to_string().as_str(),
            self.location.to_string().as_str(),
            self.publisher_name.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for ComputePublisherImageOfferListRequest {
    type Error = eyre::Report;
    type Output = Vec<ComputePublisherVmImageOfferId>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = format!(
                "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/publishers/{publisher_name}/artifacttypes/vmimage/offers?api-version=2024-07-01",
                subscription_id = self.subscription_id,
                location = self.location,
                publisher_name = self.publisher_name
            );
            let rtn = RestRequest::new(http::Method::GET, &url)?
                .authentication(AuthenticationSelection::AzureBearer {
                    auth_source: self.auth_source,
                    tenant_id: self.tenant_id,
                })
                .cache(self.cache_key()?)
                .receive_cached::<Vec<ComputePublisherVmImageOfferReference>>(
                    &executor,
                    CacheOperationOwner::of::<ComputePublisherImageOfferListRequest>(),
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
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::AzureLocationName;
    use cloud_terrastodon_azure_types::ComputePublisherName;
    use cloud_terrastodon_azure_types::Slug;
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
                let subs = fetch_all_subscriptions(
                    &AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?,
                    &executor,
                )
                .await?;
                let sub = subs.first().unwrap();
                let publisher = ComputePublisherName::try_new("center-for-internet-security-inc")?;
                let offers = super::fetch_compute_publisher_image_offers(
                    sub.id,
                    AzureLocationName::CanadaCentral,
                    publisher,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    tenant_id,
                    &executor,
                )
                .await?;
                assert!(!offers.is_empty());

                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ComputePublisherImageOfferListRequest);
cloud_terrastodon_registry::register_arbitrary!(ComputePublisherImageOfferListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(ComputePublisherImageOfferListRequest => Vec<ComputePublisherVmImageOfferId>, effects = [Read]);
