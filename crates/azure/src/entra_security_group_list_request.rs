use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
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
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

/// Security-enabled Graph v1.0 [groups](https://learn.microsoft.com/graph/api/group-list?view=graph-rest-1.0) in one explicit tenant.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraSecurityGroupListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraSecurityGroupListRequest {}

impl HasCacheKey for EntraSecurityGroupListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
                "ms",
                "graph",
                "GET",
                "security_groups",
                self.authentication.tenant_id.to_string().as_str(),
            ]))?,
            valid_for: Duration::from_secs(2 * 60 * 60),
        })
    }
}

impl ExecutableRequest for EntraSecurityGroupListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<Vec<Arc<EntraGroup>>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let key = self.cache_key()?;
            let selection = self.authentication.into();
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
            let query = MicrosoftGraphHelper::new("https://graph.microsoft.com/v1.0/groups?$filter=securityEnabled eq true", Some(key), self.authentication);
            let groups: Vec<Arc<EntraGroup>> = query.fetch_all(&executor).await?;
            debug!("Found {} security groups", groups.len());
            Ok(WorkOutput::new(groups, ArtifactObservation::now()))
                },
            )?.await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraSecurityGroupListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraSecurityGroupListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraSecurityGroupListRequest => Arc<Vec<Arc<EntraGroup>>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraSecurityGroupListRequest,
    name = "security-group-list",
    cached
);
