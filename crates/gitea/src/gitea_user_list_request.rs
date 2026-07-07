use crate::GiteaInstanceUrl;
use crate::GiteaUser;
use crate::gitea_api_support::gitea_api_get_search_paged;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct GiteaUserListRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
}

pub fn fetch_all_gitea_users<'a>(tenant: &'a GiteaInstanceUrl) -> GiteaUserListRequest<'a> {
    GiteaUserListRequest {
        tenant: Cow::Borrowed(tenant),
    }
}

impl<'a> Arbitrary<'a> for GiteaUserListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaUserListRequest<'a> {
    type Output = Vec<GiteaUser>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant.as_ref())
                .join("users")
                .join("search"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get_search_paged(self.tenant.as_ref(), self.cache_key(), |page, limit| {
            format!("/users/search?page={page}&limit={limit}")
        })
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaUserListRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(GiteaUserListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaUserListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    GiteaUserListRequest<'static> => Vec<GiteaUser>,
    effects = [Read]
);
