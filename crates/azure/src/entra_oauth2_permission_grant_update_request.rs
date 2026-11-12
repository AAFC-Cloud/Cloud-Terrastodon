use crate::EntraOAuth2PermissionGrantListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::OAuth2PermissionGrantId;
use cloud_terrastodon_azure_types::OAuth2PermissionGrantUpdateBody;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use http::Method;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

#[derive(Arbitrary, Facet)]
pub struct EntraOAuth2PermissionGrantUpdateRequest {
    pub authentication: AzureBearerAuthenticationSelection,

    pub id: OAuth2PermissionGrantId,
    pub scope: String,
}

impl InvalidatesOwnCacheKey for EntraOAuth2PermissionGrantUpdateRequest {}

impl HasCacheKey for EntraOAuth2PermissionGrantUpdateRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
                "ms",
                "graph",
                "PATCH",
                "oauth2PermissionGrants",
                self.authentication.tenant_id.to_string().as_str(),
                self.id.to_string().as_str(),
            ]))?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for EntraOAuth2PermissionGrantUpdateRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = ();
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            executor
                .deferred(
                    "Finish the submitted Azure mutation and its cache cleanup",
                    |executor| async move {
                        let cache_key = self.cache_key()?;
                        info!("Updating OAuth2 permission grant {}", self.id);
                        let url = format!(
                            "https://graph.microsoft.com/v1.0/oauth2PermissionGrants/{}",
                            self.id
                        );

                        let body = OAuth2PermissionGrantUpdateBody { scope: self.scope };
                        let execution = RestRequest::new(Method::PATCH, &url)?
                            .authentication(self.authentication.into())
                            .cache(cache_key)
                            .body(
                                facet_json::to_string_pretty(&body)
                                    .map_err(|error| eyre::eyre!("{error:?}"))?,
                            )
                            .receive_raw_with_decoder(
                                |response| {
                                    response.error_for_status()?;
                                    Ok(())
                                },
                                &executor,
                            )
                            .await;
                        let invalidation = EntraOAuth2PermissionGrantListRequest {
                            authentication: self.authentication,
                        }
                        .invalidate(&executor)
                        .await;
                        match (execution, invalidation) {
                            (Ok(()), Ok(())) => Ok(()),
                            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
                            (Err(error), Err(cleanup)) => Err(error.wrap_err(format!(
                                "OAuth2 permission grant invalidation also failed: {cleanup:#}"
                            ))),
                        }
                    },
                )
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraOAuth2PermissionGrantUpdateRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraOAuth2PermissionGrantUpdateRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraOAuth2PermissionGrantUpdateRequest => (), effects = [Write]);
