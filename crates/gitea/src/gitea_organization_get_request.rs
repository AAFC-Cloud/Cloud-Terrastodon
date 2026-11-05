use crate::GiteaInstanceUrl;
use crate::GiteaOrganization;
use crate::GiteaOrganizationName;
use crate::gitea_api_support::gitea_api_get;
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

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct GiteaOrganizationGetRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub organization_name: Cow<'a, GiteaOrganizationName>,
}

pub fn fetch_gitea_organization<'a>(
    tenant: &'a GiteaInstanceUrl,
    organization_name: &'a GiteaOrganizationName,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, GiteaOrganizationGetRequest<'a>> {
    executor.execute(GiteaOrganizationGetRequest {
        tenant: Cow::Borrowed(tenant),
        organization_name: Cow::Borrowed(organization_name),
    })
}

impl<'a> Arbitrary<'a> for GiteaOrganizationGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            organization_name: Cow::Owned(GiteaOrganizationName::arbitrary(u)?),
        })
    }
}

impl InvalidatesOwnCacheKey for GiteaOrganizationGetRequest<'_> {}

impl HasCacheKey for GiteaOrganizationGetRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: tenant_cache_key_prefix(self.tenant.as_ref())?
                .join("orgs")?
                .join(self.organization_name.as_ref().as_ref())?
                .join("show")?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaOrganizationGetRequest<'_> {
    type Output = GiteaOrganization;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            gitea_api_get(
                self.tenant.as_ref(),
                &format!("/orgs/{}", self.organization_name),
                Some(self.cache_key()?),
                &executor,
            )
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(GiteaOrganizationGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaOrganizationGetRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaOrganizationGetRequest<'static> => GiteaOrganization,
    effects = [Read]
);
