use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaUsername;
use crate::gitea_api_support::gitea_api_get_paged;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct GiteaUserRepoListRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub username: Cow<'a, GiteaUsername>,
}

pub fn fetch_all_gitea_user_repositories<'a>(
    tenant: &'a GiteaInstanceUrl,
    username: &'a GiteaUsername,
) -> GiteaUserRepoListRequest<'a> {
    GiteaUserRepoListRequest {
        tenant: Cow::Borrowed(tenant),
        username: Cow::Borrowed(username),
    }
}

impl<'a> Arbitrary<'a> for GiteaUserRepoListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            username: Cow::Owned(GiteaUsername::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaUserRepoListRequest<'a> {
    type Output = Vec<GiteaRepo>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant.as_ref())
                .join("users")
                .join(self.username.as_ref().as_ref())
                .join("repos")
                .join("list"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get_paged(self.tenant.as_ref(), self.cache_key(), |page, limit| {
            format!("/users/{}/repos?page={page}&limit={limit}", self.username)
        })
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaUserRepoListRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(GiteaUserRepoListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaUserRepoListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    GiteaUserRepoListRequest<'static> => Vec<GiteaRepo>,
    effects = [Read]
);
