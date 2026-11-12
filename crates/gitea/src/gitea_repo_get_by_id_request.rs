use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaRepoId;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::gitea_api_get_best_effort;
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
pub struct GiteaRepoGetByIdRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub account_id: CloudTerrastodonAccountId,
    pub repo_id: GiteaRepoId,
}

pub async fn try_fetch_gitea_repository_by_id(
    tenant: &GiteaInstanceUrl,
    account_id: CloudTerrastodonAccountId,
    repo_id: GiteaRepoId,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<Option<GiteaRepo>> {
    let cache_key = GiteaRepoGetByIdRequest {
        account_id,
        tenant: Cow::Borrowed(tenant),
        repo_id,
    }
    .cache_key()?;
    gitea_api_get_best_effort(
        tenant,
        account_id,
        &format!("/repositories/{repo_id}"),
        Some(cache_key),
        executor,
    )
    .await
}

impl<'a> Arbitrary<'a> for GiteaRepoGetByIdRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            account_id: CloudTerrastodonAccountId::arbitrary(u)?,
            repo_id: GiteaRepoId::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for GiteaRepoGetByIdRequest<'_> {}

impl HasCacheKey for GiteaRepoGetByIdRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: tenant_cache_key_prefix(self.tenant.as_ref())?
                .join("repositories")?
                .join("by-id")?
                .join(self.repo_id.to_string())?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaRepoGetByIdRequest<'_> {
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
                &format!("/repositories/{}", self.repo_id),
                Some(self.cache_key()?),
                &executor,
            )
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(GiteaRepoGetByIdRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoGetByIdRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaRepoGetByIdRequest<'static> => GiteaRepo,
    effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    GiteaRepoGetByIdRequest<'static>,
    name = "gitea-repo-get-by-id",
    cached
);
