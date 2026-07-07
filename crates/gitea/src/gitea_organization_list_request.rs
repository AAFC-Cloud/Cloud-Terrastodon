use crate::GiteaInstanceUrl;
use crate::GiteaOrganization;
use crate::gitea_api_support::gitea_api_get_paged;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct GiteaOrganizationListRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
}

pub fn fetch_all_gitea_organizations<'a>(
    tenant: &'a GiteaInstanceUrl,
) -> GiteaOrganizationListRequest<'a> {
    GiteaOrganizationListRequest {
        tenant: Cow::Borrowed(tenant),
    }
}

impl<'a> Arbitrary<'a> for GiteaOrganizationListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaOrganizationListRequest<'a> {
    type Output = Vec<GiteaOrganization>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant.as_ref())
                .join("orgs")
                .join("list"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get_paged(self.tenant.as_ref(), self.cache_key(), |page, limit| {
            format!("/orgs?page={page}&limit={limit}")
        })
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaOrganizationListRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(GiteaOrganizationListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaOrganizationListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    GiteaOrganizationListRequest<'static> => Vec<GiteaOrganization>,
    effects = [Read]
);
