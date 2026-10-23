use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::ComputePublisherId;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use std::path::PathBuf;

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct ComputePublishersListRequest {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub subscription_id: SubscriptionId,
    pub location: AzureLocationName,
}

pub fn fetch_compute_publishers(
    subscription_id: SubscriptionId,
    location: AzureLocationName,
    cancellation: &CancellationScope,
) -> ComputePublishersListRequest {
    ComputePublishersListRequest {
        cancellation: cancellation.clone(),
        subscription_id,
        location,
    }
}

#[async_trait]
impl CacheableCommand for ComputePublishersListRequest {
    type Output = Vec<ComputePublisherId>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "vm",
            "list-publishers",
            self.subscription_id.to_string().as_str(),
            self.location.to_string().as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let url = format!(
            "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/publishers?api-version=2024-11-01",
            subscription_id = self.subscription_id,
            location = self.location
        );
        #[derive(facet::Facet)]
        struct Row {
            id: ComputePublisherId,
            // The location and name are also present but are contained within the ID so we ignore them.
        }
        let rtn = RestRequest::new(http::Method::GET, &url, &self.cancellation)?
            .cache(self.cache_key())
            .receive::<Vec<Row>>()
            .await?
            .into_iter()
            .map(|row| row.id)
            .collect();
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ComputePublishersListRequest);

#[cfg(test)]
mod test {
    use crate::fetch_all_subscriptions;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::AzureLocationName;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let subs = fetch_all_subscriptions(
                    &AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?,
                    &cancellation,
                )
                .await?;
                let sub = subs.first().unwrap();
                let publishers = crate::fetch_compute_publishers(
                    sub.id,
                    AzureLocationName::CanadaCentral,
                    &cancellation,
                )
                .await?;
                assert!(!publishers.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ComputePublishersListRequest);
cloud_terrastodon_registry::register_arbitrary!(ComputePublishersListRequest);
cloud_terrastodon_registry::register_into_future!(ComputePublishersListRequest => Vec<ComputePublisherId>, effects = [Read]);
