use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaUsername;
use crate::gitea_api_support::gitea_api_get_paged;
use crate::gitea_api_support::tenant_cache_key_prefix;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;

#[must_use = "This is a future request, you must .await it"]
pub struct GiteaUserRepoListRequest<'a> {
    pub tenant: &'a GiteaInstanceUrl,
    pub username: &'a GiteaUsername,
}

pub fn fetch_all_gitea_user_repositories<'a>(
    tenant: &'a GiteaInstanceUrl,
    username: &'a GiteaUsername,
) -> GiteaUserRepoListRequest<'a> {
    GiteaUserRepoListRequest { tenant, username }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaUserRepoListRequest<'a> {
    type Output = Vec<GiteaRepo>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant)
                .join("users")
                .join(self.username.as_ref())
                .join("repos")
                .join("list"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get_paged(self.tenant, self.cache_key(), |page, limit| {
            format!("/users/{}/repos?page={page}&limit={limit}", self.username)
        })
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaUserRepoListRequest<'a>, 'a);
