use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use facet::Facet;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

const USER_LIST_CACHE_DURATION: Duration = Duration::MAX;
const USER_LIST_PAGE_SIZE: usize = 999;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Facet)]
pub struct EntraUserListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
    #[facet(opaque, skip, default)]
    pub progress_hook: Option<crate::MicrosoftGraphProgressHook>,
}

impl<'a> Arbitrary<'a> for EntraUserListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(AzureTenantAuthContext::arbitrary(u)?),
            progress_hook: None,
        })
    }
}

#[track_caller]
pub fn fetch_all_entra_users<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EntraUserListRequest<'a>> {
    executor.execute(EntraUserListRequest {
        auth_context: Cow::Borrowed(auth_context),
        progress_hook: None,
    })
}

impl<'a> EntraUserListRequest<'a> {
    pub fn progress_hook<F>(mut self, progress_hook: F) -> Self
    where
        F: Fn(crate::MicrosoftGraphProgress) + Send + Sync + 'static,
    {
        self.progress_hook = Some(Box::new(progress_hook));
        self
    }
}

impl<'a> InvalidatesOwnCacheKey for EntraUserListRequest<'a> {}

impl<'a> HasCacheKey for EntraUserListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
                "ms",
                "graph",
                "GET",
                "users",
                self.auth_context.tenant_id.to_string().as_str(),
            ]))?,
            valid_for: USER_LIST_CACHE_DURATION,
        })
    }
}

impl<'a> ExecutableRequest for EntraUserListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraUser>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(tenant_id = %self.auth_context.tenant_id, "Fetching users");
            let mut helper = MicrosoftGraphHelper::new(
                format!(
                    "https://graph.microsoft.com/v1.0/users?$select={}&$count=true&$top={}",
                    EntraUser::SELECT,
                    USER_LIST_PAGE_SIZE,
                ),
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            )
            .with_consistency_level_eventual()?;
            helper.progress_hook = self.progress_hook;
            let users: Vec<Arc<EntraUser>> = helper.fetch_all(&executor).await?;
            debug!("Found {} users", users.len());
            Ok(users)
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraUserListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraUserListRequest<'static>);

cloud_terrastodon_registry_execution::register_executable_request!(EntraUserListRequest<'static> => Vec<Arc<EntraUser>>, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::*;
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
                let result = fetch_all_entra_users(&auth_context, &executor).await?;
                assert!(!result.is_empty());
                Ok(())
            })
            .await
    }
}
