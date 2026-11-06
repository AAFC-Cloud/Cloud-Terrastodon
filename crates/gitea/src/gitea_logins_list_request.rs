use crate::GiteaLogin;
use arbitrary::Arbitrary;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use facet::Facet;
use std::time::Duration;

/// Inspect non-secret local tea login metadata without querying a service.
#[derive(Debug, Clone, Copy, Arbitrary, Facet)]
pub struct GiteaLoginsListRequest;

pub fn list_gitea_logins(
    executor: &CloudTerrastodonExecutor,
) -> RequestExecution<'_, GiteaLoginsListRequest> {
    executor.execute(GiteaLoginsListRequest)
}

impl InvalidatesOwnCacheKey for GiteaLoginsListRequest {}

impl HasCacheKey for GiteaLoginsListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components(["tea", "logins", "list"])?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaLoginsListRequest {
    type Error = eyre::Report;
    type Output = Vec<GiteaLogin>;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            // Always refresh: login membership selects the profile for a later
            // external process, and its configuration has no cache generation.
            CommandBuilder::new(CommandKind::Gitea)
                .cancellation_safe()
                .args(["logins", "list", "--output", "json"])
                .cache(self.cache_key()?)
                .run_with_artifacts(&executor)
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(GiteaLoginsListRequest);
cloud_terrastodon_registry::register_arbitrary!(GiteaLoginsListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaLoginsListRequest => Vec<GiteaLogin>,
    effects = [Read]
);
