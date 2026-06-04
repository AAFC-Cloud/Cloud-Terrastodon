use crate::GiteaInstanceUrl;
use crate::GiteaUser;
use crate::GiteaUsername;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::tenant_cache_key_prefix;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;

#[must_use = "This is a future request, you must .await it"]
pub struct GiteaUserGetRequest<'a> {
    pub tenant: &'a GiteaInstanceUrl,
    pub username: &'a GiteaUsername,
}

pub fn fetch_gitea_user<'a>(
    tenant: &'a GiteaInstanceUrl,
    username: &'a GiteaUsername,
) -> GiteaUserGetRequest<'a> {
    GiteaUserGetRequest { tenant, username }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaUserGetRequest<'a> {
    type Output = GiteaUser;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant)
                .join("users")
                .join(self.username.as_ref())
                .join("show"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get(
            self.tenant,
            &format!("/users/{}", self.username),
            Some(self.cache_key()),
        )
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaUserGetRequest<'a>, 'a);
