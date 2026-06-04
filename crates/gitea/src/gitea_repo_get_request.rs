use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaRepoFullName;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::tenant_cache_key_prefix;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;

#[must_use = "This is a future request, you must .await it"]
pub struct GiteaRepoGetRequest<'a> {
    pub tenant: &'a GiteaInstanceUrl,
    pub repo_full_name: &'a GiteaRepoFullName,
}

pub fn fetch_gitea_repository<'a>(
    tenant: &'a GiteaInstanceUrl,
    repo_full_name: &'a GiteaRepoFullName,
) -> GiteaRepoGetRequest<'a> {
    GiteaRepoGetRequest {
        tenant,
        repo_full_name,
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaRepoGetRequest<'a> {
    type Output = GiteaRepo;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant)
                .join("repositories")
                .join("show")
                .join(self.repo_full_name.owner.as_ref())
                .join(self.repo_full_name.repo_name.as_ref()),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get(
            self.tenant,
            &format!(
                "/repos/{}/{}",
                self.repo_full_name.owner, self.repo_full_name.repo_name
            ),
            Some(self.cache_key()),
        )
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaRepoGetRequest<'a>, 'a);
