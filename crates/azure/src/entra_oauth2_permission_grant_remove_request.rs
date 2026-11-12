use crate::EntraOAuth2PermissionGrantListRequest;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::OAuth2PermissionGrantId;
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

#[derive(Facet)]
pub struct EntraOAuth2PermissionGrantRemoveRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub id: OAuth2PermissionGrantId,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraOAuth2PermissionGrantRemoveRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
            id: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for EntraOAuth2PermissionGrantRemoveRequest {}

impl HasCacheKey for EntraOAuth2PermissionGrantRemoveRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
                "ms",
                "graph",
                "DELETE",
                "oauth2PermissionGrants",
                self.authentication.tenant_id.to_string().as_str(),
                self.id.to_string().as_str(),
            ]))?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for EntraOAuth2PermissionGrantRemoveRequest {
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
                        let url = format!(
                            "https://graph.microsoft.com/v1.0/oauth2PermissionGrants/{}",
                            self.id
                        );
                        let execution = RestRequest::new(Method::DELETE, &url)?
                            .authentication(self.authentication.into())
                            .cache(cache_key)
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

cloud_terrastodon_registry::register_thing!(EntraOAuth2PermissionGrantRemoveRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraOAuth2PermissionGrantRemoveRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraOAuth2PermissionGrantRemoveRequest => (), effects = [Write]);
