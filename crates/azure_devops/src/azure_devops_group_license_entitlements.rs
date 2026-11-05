use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupLicenseEntitlement;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsGroupLicenseEntitlementsRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

pub fn fetch_azure_devops_group_license_entitlements<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    auth_context: &'a AzureDevOpsAuthContext,
    cancellation: &CancellationScope,
) -> AzureDevOpsGroupLicenseEntitlementsRequest<'a> {
    AzureDevOpsGroupLicenseEntitlementsRequest {
        cancellation: cancellation.clone(),
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupLicenseEntitlementsRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: CancellationScope::arbitrary(u)?,
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand
    for AzureDevOpsGroupLicenseEntitlementsRequest<'a>
{
    type Output = Vec<AzureDevOpsGroupLicenseEntitlement>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "license",
            "entitlement",
            "list-groups",
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        // Endpoint and collection contract: https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/group-entitlements/list?view=azure-devops-rest-7.1
        let url = self.org_url.api_url(
            "vsaex.dev.azure.com",
            "_apis/groupentitlements",
            &[("api-version", "7.1")],
        )?;
        let response = RestRequest::from_method_and_url(Method::GET, url, &self.cancellation)?
            .cache(self.cache_key())
            .azure_devops_auth_context(self.auth_context.as_ref())?
            .receive::<AzureDevOpsCollectionResponse<Vec<AzureDevOpsGroupLicenseEntitlement>>>()
            .await?;
        Ok(response.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsGroupLicenseEntitlementsRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupLicenseEntitlementsRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsGroupLicenseEntitlementsRequest<'static>
);
cloud_terrastodon_registry_execution::register_into_future!(AzureDevOpsGroupLicenseEntitlementsRequest<'static> => Vec<AzureDevOpsGroupLicenseEntitlement>, effects = [Read]);

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
                let org_url = get_default_organization_url(&cancellation).await?;
                let entitlements =
                    fetch_azure_devops_group_license_entitlements(&org_url, &azure_devops_auth_context, &cancellation).await?;
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
