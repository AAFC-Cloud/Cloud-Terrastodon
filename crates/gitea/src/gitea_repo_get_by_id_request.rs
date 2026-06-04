use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaRepoId;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::gitea_api_get_best_effort;
use crate::gitea_api_support::tenant_cache_key_prefix;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;

#[must_use = "This is a future request, you must .await it"]
pub struct GiteaRepoGetByIdRequest<'a> {
    pub tenant: &'a GiteaInstanceUrl,
    pub repo_id: GiteaRepoId,
}

pub fn fetch_gitea_repository_by_id<'a>(
    tenant: &'a GiteaInstanceUrl,
    repo_id: GiteaRepoId,
) -> GiteaRepoGetByIdRequest<'a> {
    GiteaRepoGetByIdRequest { tenant, repo_id }
}

pub async fn try_fetch_gitea_repository_by_id(
    tenant: &GiteaInstanceUrl,
    repo_id: GiteaRepoId,
) -> eyre::Result<Option<GiteaRepo>> {
    let cache_key = CacheKey::new(
        tenant_cache_key_prefix(tenant)
            .join("repositories")
            .join("by-id")
            .join(repo_id.to_string()),
    );
    gitea_api_get_best_effort(tenant, &format!("/repositories/{repo_id}"), Some(cache_key)).await
}

#[async_trait]
impl<'a> CacheableCommand for GiteaRepoGetByIdRequest<'a> {
    type Output = GiteaRepo;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant)
                .join("repositories")
                .join("by-id")
                .join(self.repo_id.to_string()),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get(
            self.tenant,
            &format!("/repositories/{}", self.repo_id),
            Some(self.cache_key()),
        )
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaRepoGetByIdRequest<'a>, 'a);
