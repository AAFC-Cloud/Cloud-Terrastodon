use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::gitea_api_support::gitea_api_get_search_paged;
use crate::gitea_api_support::tenant_cache_key_prefix;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;

#[must_use = "This is a future request, you must .await it"]
pub struct GiteaRepoSearchRequest<'a> {
    pub tenant: &'a GiteaInstanceUrl,
}

pub fn fetch_all_gitea_repositories_via_search<'a>(
    tenant: &'a GiteaInstanceUrl,
) -> GiteaRepoSearchRequest<'a> {
    GiteaRepoSearchRequest { tenant }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaRepoSearchRequest<'a> {
    type Output = Vec<GiteaRepo>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant)
                .join("repositories")
                .join("search"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get_search_paged(self.tenant, self.cache_key(), |page, limit| {
            format!("/repos/search?page={page}&limit={limit}")
        })
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaRepoSearchRequest<'a>, 'a);
