use crate::GiteaInstanceUrl;
use crate::GiteaOrganizationName;
use crate::GiteaRepo;
use crate::gitea_api_support::gitea_api_get_paged;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct GiteaOrganizationRepoListRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub organization_name: Cow<'a, GiteaOrganizationName>,
}

pub fn fetch_all_gitea_organization_repositories<'a>(
    tenant: &'a GiteaInstanceUrl,
    organization_name: &'a GiteaOrganizationName,
) -> GiteaOrganizationRepoListRequest<'a> {
    GiteaOrganizationRepoListRequest {
        tenant: Cow::Borrowed(tenant),
        organization_name: Cow::Borrowed(organization_name),
    }
}

impl<'a> Arbitrary<'a> for GiteaOrganizationRepoListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            organization_name: Cow::Owned(GiteaOrganizationName::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaOrganizationRepoListRequest<'a> {
    type Output = Vec<GiteaRepo>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant.as_ref())
                .join("orgs")
                .join(self.organization_name.as_ref().as_ref())
                .join("repos")
                .join("list"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get_paged(self.tenant.as_ref(), self.cache_key(), |page, limit| {
            format!(
                "/orgs/{}/repos?page={page}&limit={limit}",
                self.organization_name
            )
        })
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(
    GiteaOrganizationRepoListRequest<'a>,
    'a
);

cloud_terrastodon_registry::register_thing!(GiteaOrganizationRepoListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaOrganizationRepoListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    GiteaOrganizationRepoListRequest<'static> => Vec<GiteaRepo>,
    effects = [Read]
);
