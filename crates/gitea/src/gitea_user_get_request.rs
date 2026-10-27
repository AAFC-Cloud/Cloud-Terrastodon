use crate::GiteaInstanceUrl;
use crate::GiteaUser;
use crate::GiteaUsername;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct GiteaUserGetRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub username: Cow<'a, GiteaUsername>,
}

pub fn fetch_gitea_user<'a>(
    tenant: &'a GiteaInstanceUrl,
    username: &'a GiteaUsername,
    cancellation: &CancellationScope,
) -> GiteaUserGetRequest<'a> {
    GiteaUserGetRequest {
        cancellation: cancellation.clone(),
        tenant: Cow::Borrowed(tenant),
        username: Cow::Borrowed(username),
    }
}

impl<'a> Arbitrary<'a> for GiteaUserGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: CancellationScope::unbound(),
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            username: Cow::Owned(GiteaUsername::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaUserGetRequest<'a> {
    type Output = GiteaUser;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant.as_ref())
                .join("users")
                .join(self.username.as_ref().as_ref())
                .join("show"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get(
            self.tenant.as_ref(),
            &format!("/users/{}", self.username),
            Some(self.cache_key()),
            &self.cancellation,
        )
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaUserGetRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(GiteaUserGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaUserGetRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    GiteaUserGetRequest<'static> => GiteaUser,
    effects = [Read]
);
