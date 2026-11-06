use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::gitea_api_support::gitea_api_get_search_paged;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;
use std::time::Duration;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct GiteaRepoSearchRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
}

pub fn fetch_all_gitea_repositories_via_search<'a>(
    tenant: &'a GiteaInstanceUrl,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, GiteaRepoSearchRequest<'a>> {
    executor.execute(GiteaRepoSearchRequest {
        tenant: Cow::Borrowed(tenant),
    })
}

impl<'a> Arbitrary<'a> for GiteaRepoSearchRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
        })
    }
}

impl InvalidatesOwnCacheKey for GiteaRepoSearchRequest<'_> {}

impl HasCacheKey for GiteaRepoSearchRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: tenant_cache_key_prefix(self.tenant.as_ref())?
                .join("repositories")?
                .join("search")?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaRepoSearchRequest<'_> {
    type Error = eyre::Report;
    type Output = Vec<GiteaRepo>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            gitea_api_get_search_paged(
                self.tenant.as_ref(),
                self.cache_key()?,
                |page, limit| format!("/repos/search?page={page}&limit={limit}"),
                &executor,
            )
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(GiteaRepoSearchRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoSearchRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaRepoSearchRequest<'static> => Vec<GiteaRepo>,
    effects = [Read]
);
