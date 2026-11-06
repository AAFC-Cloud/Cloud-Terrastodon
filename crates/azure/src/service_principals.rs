use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraServicePrincipal;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(facet::Facet)]
pub struct ServicePrincipalListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for ServicePrincipalListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_all_service_principals<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, ServicePrincipalListRequest<'a>> {
    executor.execute(ServicePrincipalListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> InvalidatesOwnCacheKey for ServicePrincipalListRequest<'a> {}

impl<'a> HasCacheKey for ServicePrincipalListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service_principals",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for ServicePrincipalListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraServicePrincipal>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching service principals");
            let query = MicrosoftGraphHelper::new(
                "https://graph.microsoft.com/v1.0/servicePrincipals",
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            );
            let entries: Vec<Arc<EntraServicePrincipal>> = query.fetch_all(&executor).await?;
            debug!("Found {} service principals", entries.len());
            Ok(entries)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::fetch_all_service_principals;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let found = fetch_all_service_principals(&auth_context, &executor).await?;
                assert!(found.len() > 10);
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ServicePrincipalListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ServicePrincipalListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(ServicePrincipalListRequest<'static> => Vec<Arc<EntraServicePrincipal>>, effects = [Read]);
