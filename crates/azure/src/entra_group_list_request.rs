use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraGroup;
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
pub struct EntraGroupListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraGroupListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_all_groups<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EntraGroupListRequest<'a>> {
    executor.execute(EntraGroupListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl InvalidatesOwnCacheKey for EntraGroupListRequest<'_> {}

impl HasCacheKey for EntraGroupListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "groups",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraGroupListRequest<'_> {
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraGroup>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(tenant_id = %self.auth_context.tenant_id, "Fetching Entra groups");
            let rtn: Vec<Arc<EntraGroup>> = MicrosoftGraphHelper::new(
                "https://graph.microsoft.com/v1.0/groups",
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            )
            .fetch_all(&executor)
            .await?;
            debug!(tenant_id = %self.auth_context.tenant_id, count = rtn.len(), "Found Entra groups");
            Ok(rtn)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::Result;

    #[tokio::test]
    async fn list_groups() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let auth_context =
                    AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?;
                let result = fetch_all_groups(&auth_context, &executor).await?;
                assert!(!result.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupListRequest<'static> => Vec<Arc<EntraGroup>>, effects = [Read]);
