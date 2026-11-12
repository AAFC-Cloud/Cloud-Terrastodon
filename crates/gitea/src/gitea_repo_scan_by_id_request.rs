use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaRepoId;
use crate::gitea_api_support::dedupe_repositories;
use crate::try_fetch_gitea_repository_by_id;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::borrow::Cow;
use std::time::Duration;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct GiteaRepoScanByIdRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub account_id: CloudTerrastodonAccountId,
    pub start_id: u64,
    pub end_id: u64,
}

impl<'a> Arbitrary<'a> for GiteaRepoScanByIdRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            account_id: CloudTerrastodonAccountId::arbitrary(u)?,
            start_id: u64::arbitrary(u)?,
            end_id: u64::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for GiteaRepoScanByIdRequest<'_> {}

impl HasCacheKey for GiteaRepoScanByIdRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "tea",
                self.tenant.storage_key().as_str(),
                "repositories",
                "scan-by-id",
                self.start_id.to_string().as_str(),
                self.end_id.to_string().as_str(),
            ])?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaRepoScanByIdRequest<'_> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&AuthenticationSelection::Gitea {
            account_id: self.account_id,
        })
    }

    type Error = eyre::Report;
    type Output = Vec<GiteaRepo>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let mut repositories = Vec::new();
            for repo_id in self.start_id..=self.end_id {
                executor.cancellation().bail_if_cancelled()?;
                if let Some(repo) = try_fetch_gitea_repository_by_id(
                    self.tenant.as_ref(),
                    self.account_id,
                    GiteaRepoId::new(repo_id),
                    &executor,
                )
                .await?
                {
                    repositories.push(repo);
                }
            }
            executor.cancellation().bail_if_cancelled()?;
            Ok(dedupe_repositories(repositories))
        })
    }
}

cloud_terrastodon_registry::register_thing!(GiteaRepoScanByIdRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoScanByIdRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaRepoScanByIdRequest<'static> => Vec<GiteaRepo>,
    effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    GiteaRepoScanByIdRequest<'static>,
    name = "gitea-repo-scan-by-id",
    cached
);
