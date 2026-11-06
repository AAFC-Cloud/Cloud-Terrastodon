use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
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
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsGroupLicenseEntitlementsRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_azure_devops_group_license_entitlements<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsGroupLicenseEntitlementsRequest<'a>> {
    executor.execute(AzureDevOpsGroupLicenseEntitlementsRequest {
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupLicenseEntitlementsRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
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
            .azure_devops_auth_context(self.auth_context.as_ref())?
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::get_default_organization_url;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    crate::test_tenant_id::test_tenant_id()?,
                )?;
                let executor = crate::test_executor::test_executor(&azure_devops_auth_context, &cancellation)?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let entitlements =
                    fetch_azure_devops_group_license_entitlements(&org_url, &azure_devops_auth_context, &executor).await?;
                assert!(
                    !entitlements.is_empty(),
                    "Expected at least one Azure DevOps group license entitlement"
                );
                assert!(
                    entitlements.iter().all(|entitlement| {
                        !entitlement.group.display_name.is_empty()
                            && !entitlement.group.origin_id.is_empty()
                    }),
                    "Expected sampled Azure DevOps group entitlements to include group identity data"
                );

                Ok(())

            })
            .await
    }
}
