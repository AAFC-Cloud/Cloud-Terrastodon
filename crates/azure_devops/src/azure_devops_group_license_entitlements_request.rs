use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupLicenseEntitlement;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsGroupLicenseEntitlementsRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupLicenseEntitlementsRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsGroupLicenseEntitlementsRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "license",
            "entitlement",
            "list-groups",
        ]))
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsGroupLicenseEntitlementsRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsGroupLicenseEntitlementsRequest<'a> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.validate_authentication_target(
            &self.auth_selection.into(),
            &self.org_url.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.auth_selection.into())
    }

    type Error = eyre::Report;
    type Output = Arc<Vec<AzureDevOpsGroupLicenseEntitlement>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            // Endpoint and collection contract: https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/group-entitlements/list?view=azure-devops-rest-7.1
            let url = self.org_url.api_url(
                "vsaex.dev.azure.com",
                "_apis/groupentitlements",
                &[("api-version", "7.1")],
            )?;
            let response = RestRequest::from_method_and_url(Method::GET, url)?
            .cache(self.cache_key()?)
            .authentication(self.auth_selection.into())
            .receive_cached::<AzureDevOpsCollectionResponse<Arc<Vec<AzureDevOpsGroupLicenseEntitlement>>>>(&executor, CacheOperationOwner::of::<AzureDevOpsGroupLicenseEntitlementsRequest<'static>>())
            .await?;
            Ok(Arc::clone(&response.value))
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupLicenseEntitlementsRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsGroupLicenseEntitlementsRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsGroupLicenseEntitlementsRequest<'static> => Arc<Vec<AzureDevOpsGroupLicenseEntitlement>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsGroupLicenseEntitlementsRequest<'static>,
    name = "azure-devops-group-license-entitlements",
    cached
);
