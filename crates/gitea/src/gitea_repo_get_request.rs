use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaRepoFullName;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct GiteaRepoGetRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub repo_full_name: Cow<'a, GiteaRepoFullName>,
}

pub fn fetch_gitea_repository<'a>(
    tenant: &'a GiteaInstanceUrl,
    repo_full_name: &'a GiteaRepoFullName,
) -> GiteaRepoGetRequest<'a> {
    GiteaRepoGetRequest {
        tenant: Cow::Borrowed(tenant),
        repo_full_name: Cow::Borrowed(repo_full_name),
    }
}

impl<'a> Arbitrary<'a> for GiteaRepoGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            repo_full_name: Cow::Owned(GiteaRepoFullName::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaRepoGetRequest<'a> {
    type Output = GiteaRepo;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant.as_ref())
                .join("repositories")
                .join("show")
                .join(self.repo_full_name.owner.as_ref())
                .join(self.repo_full_name.repo_name.as_ref()),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get(
            self.tenant.as_ref(),
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

cloud_terrastodon_registry::register_thing!(GiteaRepoGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoGetRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    GiteaRepoGetRequest<'static> => GiteaRepo,
    effects = [Read]
);
