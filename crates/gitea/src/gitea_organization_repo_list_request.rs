use crate::GiteaInstanceUrl;
use crate::GiteaOrganizationName;
use crate::GiteaRepo;
use crate::gitea_api_support::gitea_api_get_paged;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;
use std::time::Duration;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct GiteaOrganizationRepoListRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub organization_name: Cow<'a, GiteaOrganizationName>,
}

pub fn fetch_all_gitea_organization_repositories<'a>(
    tenant: &'a GiteaInstanceUrl,
    organization_name: &'a GiteaOrganizationName,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, GiteaOrganizationRepoListRequest<'a>> {
    executor.execute(GiteaOrganizationRepoListRequest {
        tenant: Cow::Borrowed(tenant),
        organization_name: Cow::Borrowed(organization_name),
    })
}

impl<'a> Arbitrary<'a> for GiteaOrganizationRepoListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            organization_name: Cow::Owned(GiteaOrganizationName::arbitrary(u)?),
        })
    }
}

impl InvalidatesOwnCacheKey for GiteaOrganizationRepoListRequest<'_> {}

impl HasCacheKey for GiteaOrganizationRepoListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: tenant_cache_key_prefix(self.tenant.as_ref())?
                .join("orgs")?
                .join(self.organization_name.as_ref().as_ref())?
                .join("repos")?
                .join("list")?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaOrganizationRepoListRequest<'_> {
    type Error = eyre::Report;
    type Output = Vec<GiteaRepo>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            gitea_api_get_paged(
                self.tenant.as_ref(),
                self.cache_key()?,
                |page, limit| {
                    format!(
                        "/orgs/{}/repos?page={page}&limit={limit}",
                        self.organization_name
                    )
                },
                &executor,
            )
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(GiteaOrganizationRepoListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaOrganizationRepoListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaOrganizationRepoListRequest<'static> => Vec<GiteaRepo>,
    effects = [Read]
);
