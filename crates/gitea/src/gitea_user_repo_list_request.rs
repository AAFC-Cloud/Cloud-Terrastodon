use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaUsername;
use crate::gitea_api_support::gitea_api_get_paged;
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

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct GiteaUserRepoListRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub username: Cow<'a, GiteaUsername>,
}

pub fn fetch_all_gitea_user_repositories<'a>(
    tenant: &'a GiteaInstanceUrl,
    username: &'a GiteaUsername,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, GiteaUserRepoListRequest<'a>> {
    executor.execute(GiteaUserRepoListRequest {
        tenant: Cow::Borrowed(tenant),
        username: Cow::Borrowed(username),
    })
}

impl<'a> Arbitrary<'a> for GiteaUserRepoListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            username: Cow::Owned(GiteaUsername::arbitrary(u)?),
        })
    }
}

impl InvalidatesOwnCacheKey for GiteaUserRepoListRequest<'_> {}

impl HasCacheKey for GiteaUserRepoListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: tenant_cache_key_prefix(self.tenant.as_ref())?
                .join("users")?
                .join(self.username.as_ref().as_ref())?
                .join("repos")?
                .join("list")?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaUserRepoListRequest<'_> {
    type Output = Vec<GiteaRepo>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            gitea_api_get_paged(
                self.tenant.as_ref(),
                self.cache_key()?,
                |page, limit| format!("/users/{}/repos?page={page}&limit={limit}", self.username),
                &executor,
            )
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(GiteaUserRepoListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaUserRepoListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaUserRepoListRequest<'static> => Vec<GiteaRepo>,
    effects = [Read]
);
