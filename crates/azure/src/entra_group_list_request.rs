use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
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
use tracing::debug;

/// Complete Microsoft Graph v1.0 [group inventory](https://learn.microsoft.com/graph/api/group-list?view=graph-rest-1.0).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraGroupListRequest {
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
}

#[track_caller]
pub fn fetch_all_groups<'a>(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EntraGroupListRequest> {
    executor.execute(EntraGroupListRequest {
        tenant_id,
        auth_source,
    })
}

impl InvalidatesOwnCacheKey for EntraGroupListRequest {}

impl HasCacheKey for EntraGroupListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "groups",
            self.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraGroupListRequest {
    type Error = eyre::Report;
    type Output = Arc<Vec<Arc<EntraGroup>>>;
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
            executor.cached_work(
                ExecutorKind::InProcess,
                key.clone(),
                CacheOperationOwner::of::<Self>(),
                &selection,
                fingerprint,
                WorkReuse::ReusableRead,
                move |executor| async move {
            debug!(tenant_id = %self.tenant_id, "Fetching Entra groups");
            let rtn: Vec<Arc<EntraGroup>> = MicrosoftGraphHelper::new(
                "https://graph.microsoft.com/v1.0/groups",
                Some(key),
                self.tenant_id,
                self.auth_source,
            )
            .fetch_all(&executor)
            .await?;
            debug!(tenant_id = %self.tenant_id, count = rtn.len(), "Found Entra groups");
            Ok(WorkOutput::new(rtn, ArtifactObservation::now()))
                },
            )?.await
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
                let result = fetch_all_groups(
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

cloud_terrastodon_registry::register_thing!(EntraGroupListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupListRequest => Arc<Vec<Arc<EntraGroup>>>, effects = [Read]);
