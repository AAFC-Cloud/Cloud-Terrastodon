use crate::GiteaInstanceUrl;
use crate::GiteaOrganization;
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
pub struct GiteaOrganizationListRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
}

pub fn fetch_all_gitea_organizations<'a>(
    tenant: &'a GiteaInstanceUrl,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, GiteaOrganizationListRequest<'a>> {
    executor.execute(GiteaOrganizationListRequest {
        tenant: Cow::Borrowed(tenant),
    })
}

impl<'a> Arbitrary<'a> for GiteaOrganizationListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
        })
    }
}

impl InvalidatesOwnCacheKey for GiteaOrganizationListRequest<'_> {}

impl HasCacheKey for GiteaOrganizationListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: tenant_cache_key_prefix(self.tenant.as_ref())?
                .join("orgs")?
                .join("list")?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaOrganizationListRequest<'_> {
    type Error = eyre::Report;
    type Output = Vec<GiteaOrganization>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            gitea_api_get_paged(
                self.tenant.as_ref(),
                self.cache_key()?,
                |page, limit| format!("/orgs?page={page}&limit={limit}"),
                &executor,
            )
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(GiteaOrganizationListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaOrganizationListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaOrganizationListRequest<'static> => Vec<GiteaOrganization>,
    effects = [Read]
);
