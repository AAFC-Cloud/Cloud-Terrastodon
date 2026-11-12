use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaRepoFullName;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_cache::CacheKey;
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
pub struct GiteaRepoGetRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub account_id: CloudTerrastodonAccountId,
    pub repo_full_name: Cow<'a, GiteaRepoFullName>,
}

impl<'a> Arbitrary<'a> for GiteaRepoGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            account_id: CloudTerrastodonAccountId::arbitrary(u)?,
            repo_full_name: Cow::Owned(GiteaRepoFullName::arbitrary(u)?),
        })
    }
}

impl InvalidatesOwnCacheKey for GiteaRepoGetRequest<'_> {}

impl HasCacheKey for GiteaRepoGetRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: tenant_cache_key_prefix(self.tenant.as_ref())?
                .join("repositories")?
                .join("show")?
                .join(self.repo_full_name.owner.as_ref())?
                .join(self.repo_full_name.repo_name.as_ref())?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaRepoGetRequest<'_> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&AuthenticationSelection::Gitea {
            account_id: self.account_id,
        })
    }

    type Error = eyre::Report;
    type Output = GiteaRepo;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            gitea_api_get(
                self.tenant.as_ref(),
                self.account_id,
                &format!(
                    "/repos/{}/{}",
                    self.repo_full_name.owner, self.repo_full_name.repo_name
                ),
                Some(self.cache_key()?),
                &executor,
            )
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(GiteaRepoGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoGetRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaRepoGetRequest<'static> => GiteaRepo,
    effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    GiteaRepoGetRequest<'static>,
    name = "gitea-repo-get",
    cached
);
