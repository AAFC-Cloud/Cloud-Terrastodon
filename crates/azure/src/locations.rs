use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureLocation;
use cloud_terrastodon_azure_types::AzureLocationListResponse;
use cloud_terrastodon_azure_types::AzureTenantId;
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

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct LocationListRequest {
    pub auth_source: AzureBearerAuthSource,
    pub tenant_id: AzureTenantId,

    pub subscription_id: SubscriptionId,
}

#[track_caller]
pub fn fetch_all_locations<'exec>(
    subscription_id: SubscriptionId,
    auth_source: AzureBearerAuthSource,
    tenant_id: AzureTenantId,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, LocationListRequest> {
    executor.execute(LocationListRequest {
        auth_source,
        tenant_id,

        subscription_id,
    })
}

impl InvalidatesOwnCacheKey for LocationListRequest {}

impl HasCacheKey for LocationListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "account",
            "list-locations",
            "--subscription",
            &self.subscription_id.to_string(),
        ]))
    }
}

impl ExecutableRequest for LocationListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureLocation>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = format!(
                "https://management.azure.com/subscriptions/{}/locations?api-version=2022-12-01",
                self.subscription_id
            );
            let rtn = RestRequest::new(http::Method::GET, &url)?
                .authentication(AuthenticationSelection::AzureBearer {
                    auth_source: self.auth_source,
                    tenant_id: self.tenant_id,
                })
                .cache(self.cache_key()?)
                .receive_cached::<AzureLocationListResponse>(
                    &executor,
                    CacheOperationOwner::of::<LocationListRequest>(),
                )
                .await?;
            Ok(rtn.value.clone())
        })
    }
}

#[cfg(test)]
mod test {
    use crate::fetch_all_locations;
    use crate::fetch_all_subscriptions;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
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
                let mut found_unrecognized_location = false;
                for sub in subs {
                    let locations = fetch_all_locations(
                        sub.id,
                        cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                        tenant_id,
                        &executor,
                    )
                    .await?;
                    found_unrecognized_location |= locations
                        .iter()
                        .any(|location| location.name.as_other().is_some());
                }
                if found_unrecognized_location {
                    eyre::bail!("One or more subscriptions had unrecognized locations");
                }
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(LocationListRequest);
cloud_terrastodon_registry::register_arbitrary!(LocationListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(LocationListRequest => Vec<Arc<AzureLocation>>, effects = [Read]);
