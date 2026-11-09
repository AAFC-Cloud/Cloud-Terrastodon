use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

const USER_LIST_CACHE_DURATION: Duration = Duration::MAX;
const USER_LIST_PAGE_SIZE: usize = 999;

/// Complete Microsoft Graph v1.0 [user inventory](https://learn.microsoft.com/graph/api/user-list?view=graph-rest-1.0).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraUserListRequest {
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
}

#[track_caller]
pub fn fetch_all_entra_users<'a>(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EntraUserListRequest> {
    executor.execute(EntraUserListRequest {
        tenant_id,
        auth_source,
    })
}

impl InvalidatesOwnCacheKey for EntraUserListRequest {}

impl HasCacheKey for EntraUserListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
                "ms",
                "graph",
                "GET",
                "users",
                self.tenant_id.to_string().as_str(),
            ]))?,
            valid_for: USER_LIST_CACHE_DURATION,
        })
    }
}

impl ExecutableRequest for EntraUserListRequest {
    type Error = eyre::Report;
    type Output = Arc<Vec<Arc<EntraUser>>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let key = self.cache_key()?;
            let selection = AuthenticationSelection::AzureBearer {
                tenant_id: self.tenant_id,
                auth_source: self.auth_source,
            };
            let fingerprint = NonEmptyString::new(facet_json::to_string(&self)?)?;
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    key.clone(),
                    CacheOperationOwner::of::<Self>(),
                    &selection,
                    fingerprint,
                    WorkReuse::ReusableRead,
                    move |executor| async move {
                        debug!(tenant_id = %self.tenant_id, "Fetching users");
                        let url = format!(
                            "https://graph.microsoft.com/v1.0/users?$select={}&$count=true&$top={}",
                            EntraUser::SELECT,
                            USER_LIST_PAGE_SIZE,
                        );
                        let helper = MicrosoftGraphHelper::new(
                            url,
                            Some(key),
                            self.tenant_id,
                            self.auth_source,
                        )
                        .with_consistency_level_eventual()?;
                        let users: Vec<Arc<EntraUser>> = helper.fetch_all(&executor).await?;
                        debug!("Found {} users", users.len());
                        Ok(WorkOutput::new(users, ArtifactObservation::now()))
                    },
                )?
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraUserListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUserListRequest);

cloud_terrastodon_registry_execution::register_executable_request!(EntraUserListRequest => Arc<Vec<Arc<EntraUser>>>, effects = [Read]);

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
                let result = fetch_all_entra_users(
                    auth_context.tenant_id,
                    auth_context.auth_source()?,
                    &executor,
                )
                .await?;
                assert!(!result.is_empty());
                Ok(())
            })
            .await
    }
}
