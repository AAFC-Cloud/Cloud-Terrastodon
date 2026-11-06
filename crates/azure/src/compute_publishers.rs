use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ComputePublisherId;
use cloud_terrastodon_azure_types::ComputePublisherReference;
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
pub struct ComputePublishersListRequest {
    pub auth_source: AzureBearerAuthSource,
    pub tenant_id: AzureTenantId,

    pub subscription_id: SubscriptionId,
    pub location: AzureLocationName,
}

#[track_caller]
pub fn fetch_compute_publishers<'exec>(
    subscription_id: SubscriptionId,
    location: AzureLocationName,
    auth_source: AzureBearerAuthSource,
    tenant_id: AzureTenantId,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, ComputePublishersListRequest> {
    executor.execute(ComputePublishersListRequest {
        auth_source,
        tenant_id,

        subscription_id,
        location,
    })
}

impl InvalidatesOwnCacheKey for ComputePublishersListRequest {}

impl HasCacheKey for ComputePublishersListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "vm",
            "list-publishers",
            self.subscription_id.to_string().as_str(),
            self.location.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for ComputePublishersListRequest {
    type Error = eyre::Report;
    type Output = Vec<ComputePublisherId>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = format!(
                "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/publishers?api-version=2024-11-01",
                subscription_id = self.subscription_id,
                location = self.location
            );
            let rtn = RestRequest::new(http::Method::GET, &url)?
                .authentication(AuthenticationSelection::AzureBearer {
                    auth_source: self.auth_source,
                    tenant_id: self.tenant_id,
                })
                .cache(self.cache_key()?)
                .receive_cached::<Vec<ComputePublisherReference>>(
                    &executor,
                    CacheOperationOwner::of::<ComputePublishersListRequest>(),
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
                let publishers = crate::fetch_compute_publishers(
                    sub.id,
                    AzureLocationName::CanadaCentral,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    tenant_id,
                    &executor,
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
cloud_terrastodon_registry_execution::register_executable_request!(ComputePublishersListRequest => Vec<ComputePublisherId>, effects = [Read]);
