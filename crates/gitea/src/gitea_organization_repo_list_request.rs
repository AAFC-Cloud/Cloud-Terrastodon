use crate::GiteaInstanceUrl;
use crate::GiteaOrganizationName;
use crate::GiteaRepo;
use crate::gitea_api_support::gitea_api_get_paged;
use crate::gitea_api_support::tenant_cache_key_prefix;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;

#[must_use = "This is a future request, you must .await it"]
pub struct GiteaOrganizationRepoListRequest<'a> {
    pub tenant: &'a GiteaInstanceUrl,
    pub organization_name: &'a GiteaOrganizationName,
}

pub fn fetch_all_gitea_organization_repositories<'a>(
    tenant: &'a GiteaInstanceUrl,
    organization_name: &'a GiteaOrganizationName,
) -> GiteaOrganizationRepoListRequest<'a> {
    GiteaOrganizationRepoListRequest {
        tenant,
        organization_name,
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaOrganizationRepoListRequest<'a> {
    type Output = Vec<GiteaRepo>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant)
                .join("orgs")
                .join(self.organization_name.as_ref())
                .join("repos")
                .join("list"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get_paged(self.tenant, self.cache_key(), |page, limit| {
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
