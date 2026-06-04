use crate::GiteaInstanceUrl;
use crate::GiteaUser;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::tenant_cache_key_prefix;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;

#[must_use = "This is a future request, you must .await it"]
pub struct GiteaUserCurrentGetRequest<'a> {
    pub tenant: &'a GiteaInstanceUrl,
}

pub fn fetch_current_gitea_user<'a>(
    tenant: &'a GiteaInstanceUrl,
) -> GiteaUserCurrentGetRequest<'a> {
    GiteaUserCurrentGetRequest { tenant }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaUserCurrentGetRequest<'a> {
    type Output = GiteaUser;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant)
                .join("user")
                .join("current"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get(self.tenant, "/user", Some(self.cache_key())).await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaUserCurrentGetRequest<'a>, 'a);
