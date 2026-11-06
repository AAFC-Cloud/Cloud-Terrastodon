use crate::bust_oauth2_permission_grants_cache;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ConsentType;
use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_azure_types::OAuth2PermissionGrant;
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
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
pub struct OAuth2PermissionGrantCreateRequest {
    pub auth_source: AzureBearerAuthSource,

    pub tenant_id: AzureTenantId,
    pub resource_id: EntraServicePrincipalObjectId,
    pub client_id: EntraServicePrincipalObjectId,
    pub principal_id: EntraUserId,
    pub scope: String,
}

#[track_caller]
pub fn create_oauth2_permission_grant<'exec>(
    tenant_id: AzureTenantId,
    resource_id: EntraServicePrincipalObjectId,
    client_id: EntraServicePrincipalObjectId,
    principal_id: EntraUserId,
    scope: String,
    auth_source: AzureBearerAuthSource,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, OAuth2PermissionGrantCreateRequest> {
    executor.execute(OAuth2PermissionGrantCreateRequest {
        auth_source,

        tenant_id,
        resource_id,
        client_id,
        principal_id,
        scope,
    })
}

impl InvalidatesOwnCacheKey for OAuth2PermissionGrantCreateRequest {}

impl HasCacheKey for OAuth2PermissionGrantCreateRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
                "ms",
                "graph",
                "POST",
                "oauth2PermissionGrants",
                self.tenant_id.to_string().as_str(),
                self.resource_id.to_string().as_str(),
                self.client_id.to_string().as_str(),
                self.principal_id.to_string().as_str(),
            ]))?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for OAuth2PermissionGrantCreateRequest {
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
                            .authentication(AuthenticationSelection::AzureBearer {
                                auth_source: self.auth_source,
                                tenant_id: self.tenant_id,
                            })
                            .cache(cache_key)
                            .body(
                                facet_json::to_string_pretty(&body)
                                    .map_err(|error| eyre::eyre!("{error:?}"))?,
                            )
                            .receive::<OAuth2PermissionGrant>(&executor)
                            .await;
                        let invalidation = bust_oauth2_permission_grants_cache(
                            self.tenant_id,
                            self.auth_source,
                            &executor,
                        )
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

cloud_terrastodon_registry::register_thing!(OAuth2PermissionGrantCreateRequest);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionGrantCreateRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    OAuth2PermissionGrantCreateRequest => OAuth2PermissionGrant,
    effects = [Write]
);
