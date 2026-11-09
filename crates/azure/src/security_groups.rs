use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroup;
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

/// Security-enabled Graph v1.0 [groups](https://learn.microsoft.com/graph/api/group-list?view=graph-rest-1.0) in one explicit tenant.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct SecurityGroupListRequest {
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
}

#[track_caller]
pub fn fetch_all_security_groups<'a>(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, SecurityGroupListRequest> {
    executor.execute(SecurityGroupListRequest {
        tenant_id,
        auth_source,
    })
}

impl InvalidatesOwnCacheKey for SecurityGroupListRequest {}

impl HasCacheKey for SecurityGroupListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
                "ms",
                "graph",
                "GET",
                "security_groups",
                self.tenant_id.to_string().as_str(),
            ]))?,
            valid_for: Duration::from_secs(2 * 60 * 60),
        })
    }
}

impl ExecutableRequest for SecurityGroupListRequest {
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
            debug!("Fetching security groups");
            let query = MicrosoftGraphHelper::new(
                "https://graph.microsoft.com/v1.0/groups?$filter=securityEnabled eq true",
                Some(key),
                self.tenant_id,
                self.auth_source,
            );
            let groups: Vec<Arc<EntraGroup>> = query.fetch_all(&executor).await?;
            debug!("Found {} security groups", groups.len());
            Ok(WorkOutput::new(groups, ArtifactObservation::now()))
                },
            )?.await
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
                let groups = fetch_all_security_groups(
                    auth_context.tenant_id,
                    auth_context.auth_source()?,
                    &executor,
                )
                .await?;
                assert!(groups.len() > 1);
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(SecurityGroupListRequest);
cloud_terrastodon_registry::register_arbitrary!(SecurityGroupListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(SecurityGroupListRequest => Arc<Vec<Arc<EntraGroup>>>, effects = [Read]);
