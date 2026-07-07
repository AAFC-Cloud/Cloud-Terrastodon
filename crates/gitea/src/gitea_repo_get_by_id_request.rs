use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaRepoId;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::gitea_api_get_best_effort;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct GiteaRepoGetByIdRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub repo_id: GiteaRepoId,
}

pub fn fetch_gitea_repository_by_id<'a>(
    tenant: &'a GiteaInstanceUrl,
    repo_id: GiteaRepoId,
) -> GiteaRepoGetByIdRequest<'a> {
    GiteaRepoGetByIdRequest {
        tenant: Cow::Borrowed(tenant),
        repo_id,
    }
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

impl<'a> Arbitrary<'a> for GiteaRepoGetByIdRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            repo_id: GiteaRepoId::arbitrary(u)?,
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaRepoGetByIdRequest<'a> {
    type Output = GiteaRepo;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant.as_ref())
                .join("repositories")
                .join("by-id")
                .join(self.repo_id.to_string()),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get(
            self.tenant.as_ref(),
            &format!("/repositories/{}", self.repo_id),
            Some(self.cache_key()),
        )
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaRepoGetByIdRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(GiteaRepoGetByIdRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoGetByIdRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    GiteaRepoGetByIdRequest<'static> => GiteaRepo,
    effects = [Read]
);
