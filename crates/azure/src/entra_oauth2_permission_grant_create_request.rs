use crate::EntraOAuth2PermissionGrantListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ConsentType;
use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_azure_types::OAuth2PermissionGrant;
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
use tracing::info;

#[derive(Debug, Clone, Arbitrary, Facet)]
pub struct EntraOAuth2PermissionGrantCreateRequest {
    pub authentication: AzureBearerAuthenticationSelection,

    pub resource_id: EntraServicePrincipalObjectId,
    pub client_id: EntraServicePrincipalObjectId,
    pub principal_id: EntraUserId,
    pub scope: String,
}

impl InvalidatesOwnCacheKey for EntraOAuth2PermissionGrantCreateRequest {}

impl HasCacheKey for EntraOAuth2PermissionGrantCreateRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
                "ms",
                "graph",
                "POST",
                "oauth2PermissionGrants",
                self.authentication.tenant_id.to_string().as_str(),
                self.resource_id.to_string().as_str(),
                self.client_id.to_string().as_str(),
                self.principal_id.to_string().as_str(),
            ]))?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for EntraOAuth2PermissionGrantCreateRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = OAuth2PermissionGrant;
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
                        info!(
                            "Creating OAuth2 permission grant for {} for {}",
                            self.scope, self.principal_id
                        );
                        let url = "https://graph.microsoft.com/v1.0/oauth2PermissionGrants";
                        let body = OAuth2PermissionGrant {
                            resource_id: self.resource_id,
                            client_id: self.client_id,
                            consent_type: ConsentType::Principal,
                            id: OAuth2PermissionGrantId("".to_string()),
                            principal_id: Some(self.principal_id),
                            scope: self.scope,
                        };
                        let execution = RestRequest::new(Method::POST, url)?
                            .authentication(self.authentication.into())
                            .cache(cache_key)
                            .body(
                                facet_json::to_string_pretty(&body)
                                    .map_err(|error| eyre::eyre!("{error:?}"))?,
                            )
                            .receive::<OAuth2PermissionGrant>(&executor)
                            .await;
                        let invalidation = EntraOAuth2PermissionGrantListRequest {
                            authentication: self.authentication,
                        }
                        .invalidate(&executor)
                        .await;
                        match (execution, invalidation) {
                            (Ok(created), Ok(())) => Ok(created),
                            (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
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

cloud_terrastodon_registry::register_thing!(EntraOAuth2PermissionGrantCreateRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraOAuth2PermissionGrantCreateRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraOAuth2PermissionGrantCreateRequest => OAuth2PermissionGrant,
    effects = [Write]
);
