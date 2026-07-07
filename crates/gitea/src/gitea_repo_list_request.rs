use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::dedupe_repositories;
use crate::fetch_all_gitea_organization_repositories;
use crate::fetch_all_gitea_organizations;
use crate::fetch_all_gitea_repositories_via_search;
use crate::fetch_all_gitea_user_repositories;
use crate::fetch_all_gitea_users;
use crate::fetch_current_user_gitea_repositories;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct GiteaRepoListRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
}

pub fn fetch_all_gitea_repositories<'a>(tenant: &'a GiteaInstanceUrl) -> GiteaRepoListRequest<'a> {
    GiteaRepoListRequest {
        tenant: Cow::Borrowed(tenant),
    }
}

impl<'a> Arbitrary<'a> for GiteaRepoListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
        })
    }
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
        let tenant = self.tenant;
        let mut repositories = Vec::new();
        repositories.extend(fetch_all_gitea_repositories_via_search(tenant.as_ref()).await?);
        repositories.extend(fetch_current_user_gitea_repositories(tenant.as_ref()).await?);

        let organizations = fetch_all_gitea_organizations(tenant.as_ref()).await?;
        for organization in &organizations {
            repositories.extend(
                fetch_all_gitea_organization_repositories(tenant.as_ref(), &organization.username)
                    .await?,
            );
        }

        let users = fetch_all_gitea_users(tenant.as_ref()).await?;
        for user in &users {
            repositories
                .extend(fetch_all_gitea_user_repositories(tenant.as_ref(), &user.login).await?);
        }

        Ok(dedupe_repositories(repositories))
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaRepoListRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(GiteaRepoListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    GiteaRepoListRequest<'static> => Vec<GiteaRepo>,
    effects = [Read]
);
