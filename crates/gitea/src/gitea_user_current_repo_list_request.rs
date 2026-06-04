use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::gitea_api_support::gitea_api_get_paged;
use crate::gitea_api_support::tenant_cache_key_prefix;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;

#[must_use = "This is a future request, you must .await it"]
pub struct GiteaUserCurrentRepoListRequest<'a> {
    pub tenant: &'a GiteaInstanceUrl,
}

pub fn fetch_current_user_gitea_repositories<'a>(
    tenant: &'a GiteaInstanceUrl,
) -> GiteaUserCurrentRepoListRequest<'a> {
    GiteaUserCurrentRepoListRequest { tenant }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaUserCurrentRepoListRequest<'a> {
    type Output = Vec<GiteaRepo>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant)
                .join("user")
                .join("current")
                .join("repos")
                .join("list"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get_paged(self.tenant, self.cache_key(), |page, limit| {
            format!("/user/repos?page={page}&limit={limit}")
        })
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaUserCurrentRepoListRequest<'a>, 'a);
