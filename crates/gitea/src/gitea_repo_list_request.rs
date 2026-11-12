use crate::GiteaInstanceUrl;
use crate::GiteaOrganizationListRequest;
use crate::GiteaOrganizationRepoListRequest;
use crate::GiteaRepo;
use crate::GiteaRepoSearchRequest;
use crate::GiteaUserCurrentRepoListRequest;
use crate::GiteaUserListRequest;
use crate::GiteaUserRepoListRequest;
use crate::dedupe_repositories;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::borrow::Cow;
use std::time::Duration;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct GiteaRepoListRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub account_id: CloudTerrastodonAccountId,
}

impl<'a> Arbitrary<'a> for GiteaRepoListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            account_id: CloudTerrastodonAccountId::arbitrary(u)?,
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
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&AuthenticationSelection::Gitea {
            account_id: self.account_id,
        })
    }

    type Error = eyre::Report;
    type Output = Vec<GiteaRepo>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let mut repositories = Vec::new();
            repositories.extend(
                GiteaRepoSearchRequest {
                    account_id: self.account_id,
                    tenant: Cow::Borrowed(self.tenant.as_ref()),
                }
                .execute(&executor)
                .await?,
            );
            repositories.extend(
                GiteaUserCurrentRepoListRequest {
                    account_id: self.account_id,
                    tenant: Cow::Borrowed(self.tenant.as_ref()),
                }
                .execute(&executor)
                .await?,
            );

            let organizations = GiteaOrganizationListRequest {
                account_id: self.account_id,
                tenant: Cow::Borrowed(self.tenant.as_ref()),
            }
            .execute(&executor)
            .await?;
            for organization in &organizations {
                executor.cancellation().bail_if_cancelled()?;
                repositories.extend(
                    GiteaOrganizationRepoListRequest {
                        account_id: self.account_id,
                        tenant: Cow::Borrowed(self.tenant.as_ref()),
                        organization_name: Cow::Borrowed(&organization.username),
                    }
                    .execute(&executor)
                    .await?,
                );
            }

            let users = GiteaUserListRequest {
                account_id: self.account_id,
                tenant: Cow::Borrowed(self.tenant.as_ref()),
            }
            .execute(&executor)
            .await?;
            for user in &users {
                executor.cancellation().bail_if_cancelled()?;
                repositories.extend(
                    GiteaUserRepoListRequest {
                        account_id: self.account_id,
                        tenant: Cow::Borrowed(self.tenant.as_ref()),
                        username: Cow::Borrowed(&user.login),
                    }
                    .execute(&executor)
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

cloud_terrastodon_registry_execution::register_runtime_test!(
    GiteaRepoListRequest<'static>,
    name = "gitea-repo-list",
    cached
);
