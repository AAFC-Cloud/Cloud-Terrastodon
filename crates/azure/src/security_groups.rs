use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(facet::Facet)]
pub struct SecurityGroupListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for SecurityGroupListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_all_security_groups<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, SecurityGroupListRequest<'a>> {
    executor.execute(SecurityGroupListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> InvalidatesOwnCacheKey for SecurityGroupListRequest<'a> {}

impl<'a> HasCacheKey for SecurityGroupListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
                "ms",
                "graph",
                "GET",
                "security_groups",
            ]))?,
            valid_for: Duration::from_secs(2 * 60 * 60),
        })
    }
}

impl<'a> ExecutableRequest for SecurityGroupListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraGroup>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching security groups");
            let query = MicrosoftGraphHelper::new(
                "https://graph.microsoft.com/v1.0/groups?$filter=securityEnabled eq true",
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            );
            let groups: Vec<Arc<EntraGroup>> = query.fetch_all(&executor).await?;
            debug!("Found {} security groups", groups.len());
            Ok(groups)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::fetch_all_security_groups;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::Result;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let groups = fetch_all_security_groups(&auth_context, &executor).await?;
                assert!(groups.len() > 1);
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(SecurityGroupListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(SecurityGroupListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(SecurityGroupListRequest<'static> => Vec<Arc<EntraGroup>>, effects = [Read]);
