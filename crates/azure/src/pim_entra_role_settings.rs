use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
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
use http::Method;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// The selected role's legacy [PIM settings](https://learn.microsoft.com/graph/api/resources/governancerolesetting?view=graph-rest-beta).
#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct EntraPimRoleSettingsRequest {
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
    pub role_definition_id: Uuid,
}
pub fn fetch_entra_pim_role_settings(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    role_definition_id: Uuid,
) -> EntraPimRoleSettingsRequest {
    EntraPimRoleSettingsRequest {
        tenant_id,
        auth_source,
        role_definition_id,
    }
}
impl HasCacheKey for EntraPimRoleSettingsRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "graph",
                "beta",
                "tenants",
                &self.tenant_id.to_string(),
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
    type Error = eyre::Report;
    type Output = Arc<PimEntraRoleSettings>;
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let url = Url::parse(&format!(
                "https://graph.microsoft.com/beta/privilegedAccess/aadroles/resources/{}/roleSettings?$select=id,roleDefinitionId,userMemberSettings&$filter=(roleDefinition/id eq '{}')",
                self.tenant_id, self.role_definition_id,
            ))?;
            let request = RestRequest::from_method_and_url(Method::GET, url)?
                .authentication(AuthenticationSelection::AzureBearer {
                    auth_source: self.auth_source,
                    tenant_id: self.tenant_id,
                })
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

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth::CredentialAuthenticationProvider;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_command::CancellationToken;
    #[tokio::test]
    #[ignore = "requires explicit tenant configuration, credentials and live Graph access"]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|scope| async move {
                let tenant_id = crate::get_test_tenant_id(&scope).await?;
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(CredentialAuthenticationProvider::new(
                        AuthContext::explicit_azure_cli(),
                    )),
                    scope,
                );
                let Some(assignments) = crate::test_helpers::expect_aad_premium_p2_license(
                    crate::fetch_my_entra_pim_role_assignments(
                        tenant_id,
                        AzureBearerAuthSource::AzureCli,
                    )
                    .execute(&executor)
                    .await,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    tenant_id,
                    &executor,
                )
                .await?
                else {
                    return Ok(());
                };
                assert!(!assignments.is_empty());
                for assignment in assignments {
                    let settings = fetch_entra_pim_role_settings(
                        tenant_id,
                        AzureBearerAuthSource::AzureCli,
                        assignment.role_definition_id,
                    )
                    .execute(&executor)
                    .await?;
                    assert!(settings.get_maximum_grant_period()?.as_secs() % (60 * 30) == 0);
                }
                Ok(())
            })
            .await
    }
}
