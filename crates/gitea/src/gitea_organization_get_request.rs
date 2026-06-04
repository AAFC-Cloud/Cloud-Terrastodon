use crate::GiteaInstanceUrl;
use crate::GiteaOrganization;
use crate::GiteaOrganizationName;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::tenant_cache_key_prefix;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;

#[must_use = "This is a future request, you must .await it"]
pub struct GiteaOrganizationGetRequest<'a> {
    pub tenant: &'a GiteaInstanceUrl,
    pub organization_name: &'a GiteaOrganizationName,
}

pub fn fetch_gitea_organization<'a>(
    tenant: &'a GiteaInstanceUrl,
    organization_name: &'a GiteaOrganizationName,
) -> GiteaOrganizationGetRequest<'a> {
    GiteaOrganizationGetRequest {
        tenant,
        organization_name,
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaOrganizationGetRequest<'a> {
    type Output = GiteaOrganization;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant)
                .join("orgs")
                .join(self.organization_name.as_ref())
                .join("show"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get(
            self.tenant,
            &format!("/orgs/{}", self.organization_name),
            Some(self.cache_key()),
        )
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaOrganizationGetRequest<'a>, 'a);
