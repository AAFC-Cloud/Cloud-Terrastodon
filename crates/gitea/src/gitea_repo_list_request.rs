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
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
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
pub struct GiteaRepoListRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
}

pub fn fetch_all_gitea_repositories<'a>(
    tenant: &'a GiteaInstanceUrl,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, GiteaRepoListRequest<'a>> {
    executor.execute(GiteaRepoListRequest {
        tenant: Cow::Borrowed(tenant),
    })
}

impl<'a> Arbitrary<'a> for GiteaRepoListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
        })
    }
}

impl InvalidatesOwnCacheKey for GiteaRepoListRequest<'_> {}

impl HasCacheKey for GiteaRepoListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "tea",
                self.tenant.storage_key().as_str(),
                "repositories",
                "enumerate",
                "combined",
            ])?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaRepoListRequest<'_> {
    type Error = eyre::Report;
    type Output = Vec<GiteaRepo>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let mut repositories = Vec::new();
            repositories.extend(
                fetch_all_gitea_repositories_via_search(self.tenant.as_ref(), &executor).await?,
            );
            repositories.extend(
                fetch_current_user_gitea_repositories(self.tenant.as_ref(), &executor).await?,
            );

            let organizations =
                fetch_all_gitea_organizations(self.tenant.as_ref(), &executor).await?;
            for organization in &organizations {
                executor.cancellation().bail_if_cancelled()?;
                repositories.extend(
                    fetch_all_gitea_organization_repositories(
                        self.tenant.as_ref(),
                        &organization.username,
                        &executor,
                    )
                    .await?,
                );
            }

            let users = fetch_all_gitea_users(self.tenant.as_ref(), &executor).await?;
            for user in &users {
                executor.cancellation().bail_if_cancelled()?;
                repositories.extend(
                    fetch_all_gitea_user_repositories(self.tenant.as_ref(), &user.login, &executor)
                        .await?,
                );
            }

            executor.cancellation().bail_if_cancelled()?;
            Ok(dedupe_repositories(repositories))
        })
    }
}

cloud_terrastodon_registry::register_thing!(GiteaRepoListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaRepoListRequest<'static> => Vec<GiteaRepo>,
    effects = [Read]
);
