use crate::GiteaInstanceUrl;
use crate::GiteaOrganization;
use crate::GiteaOrganizationName;
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
pub struct GiteaOrganizationGetRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub organization_name: Cow<'a, GiteaOrganizationName>,
}

pub fn fetch_gitea_organization<'a>(
    tenant: &'a GiteaInstanceUrl,
    organization_name: &'a GiteaOrganizationName,
    cancellation: &CancellationScope,
) -> GiteaOrganizationGetRequest<'a> {
    GiteaOrganizationGetRequest {
        cancellation: cancellation.clone(),
        tenant: Cow::Borrowed(tenant),
        organization_name: Cow::Borrowed(organization_name),
    }
}

impl<'a> Arbitrary<'a> for GiteaOrganizationGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: CancellationScope::unbound(),
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            organization_name: Cow::Owned(GiteaOrganizationName::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaOrganizationGetRequest<'a> {
    type Output = GiteaOrganization;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant.as_ref())
                .join("orgs")
                .join(self.organization_name.as_ref().as_ref())
                .join("show"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get(
            self.tenant.as_ref(),
            &format!("/orgs/{}", self.organization_name),
            Some(self.cache_key()),
            &self.cancellation,
        )
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaOrganizationGetRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(GiteaOrganizationGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaOrganizationGetRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    GiteaOrganizationGetRequest<'static> => GiteaOrganization,
    effects = [Read]
);
