use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure_types::OAuth2PermissionScope;
use cloud_terrastodon_azure_types::OAuth2PermissionScopeListResponse;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[derive(Facet)]
pub struct OAuth2PermissionScopesListRequest<'a> {
    pub service_principal_id: EntraServicePrincipalObjectId,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for OAuth2PermissionScopesListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            service_principal_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_oauth2_permission_scopes<'a>(
    service_principal_id: EntraServicePrincipalObjectId,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, OAuth2PermissionScopesListRequest<'a>> {
    executor.execute(OAuth2PermissionScopesListRequest {
        service_principal_id,
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> InvalidatesOwnCacheKey for OAuth2PermissionScopesListRequest<'a> {}

impl<'a> HasCacheKey for OAuth2PermissionScopesListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "oauth2_permission_scopes",
            self.auth_context.tenant_id.to_string().as_ref(),
            self.service_principal_id.to_string().as_ref(),
        ]))
    }
}

impl<'a> ExecutableRequest for OAuth2PermissionScopesListRequest<'a> {
    type Error = eyre::Report;
    type Output = Arc<Vec<OAuth2PermissionScope>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            info!(
                "Fetching OAuth2 permission scopes for {:?}",
                self.service_principal_id
            );
            let url = format!(
                "https://graph.microsoft.com/v1.0/servicePrincipals/{service_principal_id}?$select=oauth2PermissionScopes",
                service_principal_id = self.service_principal_id
            );
            let entries = RestRequest::new(http::Method::GET, url.as_str())?
                .azure_auth_context(&self.auth_context)?
                .cache(self.cache_key()?)
                .receive_cached::<OAuth2PermissionScopeListResponse>(
                    &executor,
                    CacheOperationOwner::of::<OAuth2PermissionScopesListRequest<'static>>(),
                )
                .await?;

            info!(
                "Found {} OAuth2 permission scopes",
                entries.oauth2_permission_scopes.len()
            );
            Ok(Arc::clone(&entries.oauth2_permission_scopes))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_all_service_principals;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::OptionExt;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let auth_context = AuthContext::explicit_azure_cli();
                let tenant_auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
                let service_principals = fetch_all_service_principals(
                    tenant_auth_context.tenant_id,
                    tenant_auth_context.auth_source()?,
                    &executor,
                )
                .await?;
                let graph = service_principals
                    .iter()
                    .find(|sp| sp.display_name == "Microsoft Graph")
                    .ok_or_eyre("Failed to find graph sp")?;
                let scopes =
                    fetch_oauth2_permission_scopes(graph.id, &tenant_auth_context, &executor)
                        .await?;
                assert!(scopes.len() > 10);
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(OAuth2PermissionScopesListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionScopesListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(OAuth2PermissionScopesListRequest<'static> => Arc<Vec<OAuth2PermissionScope>>, effects = [Read]);
