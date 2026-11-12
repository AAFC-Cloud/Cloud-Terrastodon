use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraPimCollectionResponse;
use cloud_terrastodon_azure_types::PimEntraRoleSettings;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use http::Method;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// The selected role's legacy [PIM settings](https://learn.microsoft.com/graph/api/resources/governancerolesetting?view=graph-rest-beta).
#[derive(Arbitrary, Facet)]
pub struct EntraPimRoleSettingsRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub role_definition_id: Uuid,
}

impl HasCacheKey for EntraPimRoleSettingsRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "graph",
                "beta",
                "tenants",
                &self.authentication.tenant_id.to_string(),
                "pim",
                "role-settings",
                &self.role_definition_id.to_string(),
            ])?,
            valid_for: Duration::from_secs(3600),
        })
    }
}
impl InvalidatesOwnCacheKey for EntraPimRoleSettingsRequest {}
impl ExecutableRequest for EntraPimRoleSettingsRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<PimEntraRoleSettings>;
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let url = Url::parse(&format!(
                "https://graph.microsoft.com/beta/privilegedAccess/aadroles/resources/{}/roleSettings?$select=id,roleDefinitionId,userMemberSettings&$filter=(roleDefinition/id eq '{}')",
                self.authentication.tenant_id, self.role_definition_id,
            ))?;
            let request = RestRequest::from_method_and_url(Method::GET, url)?
                .authentication(self.authentication.into())
                .cache(self.cache_key()?);
            let validate = |response: &EntraPimCollectionResponse<PimEntraRoleSettings>| {
                eyre::ensure!(
                    response.value.len() == 1 && response.next_link.is_none(),
                    "Expected a single PIM role-settings result, got {} with continuation {}",
                    response.value.len(),
                    response.next_link.is_some()
                );
                Ok(())
            };
            let mut result = request
                .clone()
                .receive_with_validator_cached(
                    validate,
                    &executor,
                    CacheOperationOwner::of::<Self>(),
                )
                .await;
            if result.is_err() {
                executor.cancellation().bail_if_cancelled()?;
                result = request
                    .receive_with_validator_cached(
                        validate,
                        &executor,
                        CacheOperationOwner::of::<Self>(),
                    )
                    .await;
            }
            result?
                .value
                .first()
                .cloned()
                .ok_or_else(|| eyre::eyre!("PIM role settings response is empty"))
        })
    }
}
cloud_terrastodon_registry::register_thing!(EntraPimRoleSettingsRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraPimRoleSettingsRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraPimRoleSettingsRequest => Arc<PimEntraRoleSettings>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraPimRoleSettingsRequest,
    name = "entra-pim-role-settings",
    cached
);
