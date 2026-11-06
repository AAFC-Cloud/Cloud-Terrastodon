use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraPimCollectionResponse;
use cloud_terrastodon_azure_types::PimEntraRoleDefinition;
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

/// List the tenant's legacy [PIM role definitions](https://learn.microsoft.com/graph/api/resources/governanceroledefinition?view=graph-rest-beta).
#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct PimEntraRoleDefinitionListRequest {
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
}

pub fn fetch_all_entra_pim_role_definitions(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
) -> PimEntraRoleDefinitionListRequest {
    PimEntraRoleDefinitionListRequest {
        tenant_id,
        auth_source,
    }
}

impl HasCacheKey for PimEntraRoleDefinitionListRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "graph",
                "beta",
                "tenants",
                &self.tenant_id.to_string(),
                "pim",
                "role-definitions",
            ])?,
            valid_for: Duration::from_secs(3600),
        })
    }
}
impl InvalidatesOwnCacheKey for PimEntraRoleDefinitionListRequest {}
impl ExecutableRequest for PimEntraRoleDefinitionListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<PimEntraRoleDefinition>>;
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let initial_url = Url::parse(&format!(
                "https://graph.microsoft.com/beta/privilegedAccess/aadroles/resources/{}/roleDefinitions?$select=id,displayName,type,isbuiltIn&$orderby=displayName",
                self.tenant_id
            ))?;
            let mut next = Some(initial_url.clone());
            let mut visited = std::collections::HashSet::new();
            let mut values = Vec::new();
            let mut page_index = 0usize;
            while let Some(url) = next {
                executor.cancellation().bail_if_cancelled()?;
                eyre::ensure!(
                    url.origin() == initial_url.origin()
                        && url.path() == initial_url.path()
                        && url.username().is_empty()
                        && url.password().is_none()
                        && url.fragment().is_none(),
                    "Graph PIM continuation left the selected resource"
                );
                eyre::ensure!(
                    visited.insert(url.clone()),
                    "Graph PIM continuation repeated a page"
                );
                let request = RestRequest::from_method_and_url(Method::GET, url)?
                    .authentication(AuthenticationSelection::AzureBearer {
                        auth_source: self.auth_source,
                        tenant_id: self.tenant_id,
                    })
                    .cache(self.cache_key()?.join(format!("pages/{page_index}"))?);
                let mut result = request
                    .clone()
                    .receive_cached::<EntraPimCollectionResponse<PimEntraRoleDefinition>>(
                        &executor,
                        CacheOperationOwner::of::<Self>(),
                    )
                    .await;
                if result.is_err() {
                    executor.cancellation().bail_if_cancelled()?;
                    // Preserve the existing single retry for transient gateway errors.
                    result = request
                        .receive_cached(&executor, CacheOperationOwner::of::<Self>())
                        .await;
                }
                let response = result?;
                values.extend(response.value.iter().cloned());
                next = response.next_link.clone();
                page_index += 1;
            }
            Ok(values)
        })
    }
}
cloud_terrastodon_registry::register_thing!(PimEntraRoleDefinitionListRequest);
cloud_terrastodon_registry::register_arbitrary!(PimEntraRoleDefinitionListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(PimEntraRoleDefinitionListRequest => Vec<Arc<PimEntraRoleDefinition>>, effects = [Read]);

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
                let Some(result) = crate::test_helpers::expect_aad_premium_p2_license(
                    fetch_all_entra_pim_role_definitions(
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
                assert!(!result.is_empty());
                Ok(())
            })
            .await
    }
}
