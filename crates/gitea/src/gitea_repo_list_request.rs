use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::dedupe_repositories;
use crate::fetch_all_gitea_organization_repositories;
use crate::fetch_all_gitea_organizations;
use crate::fetch_all_gitea_repositories_via_search;
use crate::fetch_all_gitea_user_repositories;
use crate::fetch_all_gitea_users;
use crate::fetch_current_user_gitea_repositories;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct GiteaRepoListRequest<'a> {
    pub tenant: &'a GiteaInstanceUrl,
}

pub fn fetch_all_gitea_repositories<'a>(tenant: &'a GiteaInstanceUrl) -> GiteaRepoListRequest<'a> {
    GiteaRepoListRequest { tenant }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaRepoListRequest<'a> {
    type Output = Vec<GiteaRepo>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "tea",
            self.tenant.storage_key().as_str(),
            "repositories",
            "enumerate",
            "combined",
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let mut repositories = Vec::new();
        repositories.extend(fetch_all_gitea_repositories_via_search(self.tenant).await?);
        repositories.extend(fetch_current_user_gitea_repositories(self.tenant).await?);

        let organizations = fetch_all_gitea_organizations(self.tenant).await?;
        for organization in &organizations {
            repositories.extend(
                fetch_all_gitea_organization_repositories(self.tenant, &organization.username)
                    .await?,
            );
        }

        let users = fetch_all_gitea_users(self.tenant).await?;
        for user in &users {
            repositories.extend(fetch_all_gitea_user_repositories(self.tenant, &user.login).await?);
        }

        Ok(dedupe_repositories(repositories))
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaRepoListRequest<'a>, 'a);
