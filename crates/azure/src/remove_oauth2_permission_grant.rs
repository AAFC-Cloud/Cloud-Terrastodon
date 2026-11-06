use crate::bust_oauth2_permission_grants_cache;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::OAuth2PermissionGrantId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use http::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;

#[derive(facet::Facet)]
pub struct OAuth2PermissionGrantRemoveRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
    pub id: OAuth2PermissionGrantId,
}

impl<'a> arbitrary::Arbitrary<'a> for OAuth2PermissionGrantRemoveRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
            id: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

#[track_caller]
pub fn remove_oauth2_permission_grant<'a>(
    auth_context: &'a AzureTenantAuthContext,
    id: OAuth2PermissionGrantId,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, OAuth2PermissionGrantRemoveRequest<'a>> {
    executor.execute(OAuth2PermissionGrantRemoveRequest {
        auth_context: Cow::Borrowed(auth_context),
        id,
    })
}

impl InvalidatesOwnCacheKey for OAuth2PermissionGrantRemoveRequest<'_> {}

impl HasCacheKey for OAuth2PermissionGrantRemoveRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
                "ms",
                "graph",
                "DELETE",
                "oauth2PermissionGrants",
                self.auth_context.tenant_id.to_string().as_str(),
                self.id.to_string().as_str(),
            ]))?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for OAuth2PermissionGrantRemoveRequest<'_> {
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
                            .azure_auth_context(&self.auth_context)?
                            .cache(cache_key)
                            .receive_raw_with_decoder(
                                |response| {
                                    response.error_for_status()?;
                                    Ok(())
                                },
                                &executor,
                            )
                            .await;
                        let invalidation = bust_oauth2_permission_grants_cache(
                            self.auth_context.tenant_id,
                            self.auth_context
                                .auth_context
                                .source()
                                .ok_or_else(|| eyre::eyre!("authentication source is unresolved"))?
                                .try_into()?,
                            &executor,
                        )
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

cloud_terrastodon_registry::register_thing!(OAuth2PermissionGrantRemoveRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionGrantRemoveRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(OAuth2PermissionGrantRemoveRequest<'static> => (), effects = [Write]);
