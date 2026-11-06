use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaRepoId;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::gitea_api_get_best_effort;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use facet::Facet;
use std::borrow::Cow;
use std::time::Duration;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct GiteaRepoGetByIdRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub repo_id: GiteaRepoId,
}

pub fn fetch_gitea_repository_by_id<'a>(
    tenant: &'a GiteaInstanceUrl,
    repo_id: GiteaRepoId,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, GiteaRepoGetByIdRequest<'a>> {
    executor.execute(GiteaRepoGetByIdRequest {
        tenant: Cow::Borrowed(tenant),
        repo_id,
    })
}

pub async fn try_fetch_gitea_repository_by_id(
    tenant: &GiteaInstanceUrl,
    repo_id: GiteaRepoId,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<Option<GiteaRepo>> {
    let cache_key = GiteaRepoGetByIdRequest {
        tenant: Cow::Borrowed(tenant),
        repo_id,
    }
    .cache_key()?;
    gitea_api_get_best_effort(
        tenant,
        &format!("/repositories/{repo_id}"),
        Some(cache_key),
        executor,
    )
    .await
}

impl<'a> Arbitrary<'a> for GiteaRepoGetByIdRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            repo_id: GiteaRepoId::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for GiteaRepoGetByIdRequest<'_> {}

impl HasCacheKey for GiteaRepoGetByIdRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: tenant_cache_key_prefix(self.tenant.as_ref())?
                .join("repositories")?
                .join("by-id")?
                .join(self.repo_id.to_string())?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaRepoGetByIdRequest<'_> {
    type Error = eyre::Report;
    type Output = GiteaRepo;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            gitea_api_get(
                self.tenant.as_ref(),
                &format!("/repositories/{}", self.repo_id),
                Some(self.cache_key()?),
                &executor,
            )
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(GiteaRepoGetByIdRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoGetByIdRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaRepoGetByIdRequest<'static> => GiteaRepo,
    effects = [Read]
);
