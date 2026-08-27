use crate::azure_devops_rest::azure_devops_api_url;
use crate::azure_devops_rest::page_cache_key;
use crate::azure_devops_rest::receive_azure_devops_page;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsUserLicenseEntitlementListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
}

pub fn fetch_azure_devops_user_license_entitlements<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
) -> AzureDevOpsUserLicenseEntitlementListRequest<'a> {
    AzureDevOpsUserLicenseEntitlementListRequest {
        org_url: Cow::Borrowed(org_url),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsUserLicenseEntitlementListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand
    for AzureDevOpsUserLicenseEntitlementListRequest<'a>
{
    type Output = Vec<AzureDevOpsUserLicenseEntitlement>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "license",
            "entitlement",
            "list",
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching Azure DevOps user entitlements");
        #[derive(Facet)]
        #[facet(rename_all = "camelCase")]
        struct InvokeResponse {
            count: u32,
            value: Vec<AzureDevOpsUserLicenseEntitlement>,
        }

        let mut entitlements = Vec::new();
        let mut continuation = None;
        let mut count = 0;
        let cache_key = self.cache_key();
        let mut page_index = 0;
        loop {
            let mut query = vec![("api-version", "7.1-preview.3")];
            if let Some(token) = continuation.as_deref() {
                query.push(("continuationToken", token));
            }
            let url = azure_devops_api_url(
                &self.org_url,
                "vsaex.dev.azure.com",
                "_apis/userentitlements",
                &query,
            )?;
            let request = RestRequest::new(Method::GET, url)?;
            let (response, next_continuation) = receive_azure_devops_page::<InvokeResponse>(
                request.cache(page_cache_key(&cache_key, page_index)),
            )
            .await?;
            count += response.count;
            entitlements.extend(response.value);
            continuation = next_continuation;
            page_index += 1;
            if continuation.is_none() {
                break;
            }
        }

        debug!("Found {count} Azure DevOps user entitlements");
        Ok(entitlements)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsUserLicenseEntitlementListRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(AzureDevOpsUserLicenseEntitlementListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsUserLicenseEntitlementListRequest<'static>
);
cloud_terrastodon_registry::register_into_future!(AzureDevOpsUserLicenseEntitlementListRequest<'static> => Vec<AzureDevOpsUserLicenseEntitlement>, effects = [Read]);

#[cfg(test)]
mod test {
    use super::*;
    use crate::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let entitlements = fetch_azure_devops_user_license_entitlements(&org_url).await?;
        assert!(
            !entitlements.is_empty(),
            "Expected at least one Azure DevOps user entitlement"
        );
        assert!(
            entitlements.iter().all(|entitlement| {
                !entitlement.user.display_name.is_empty()
                    && !entitlement.user.unique_name.is_empty()
            }),
            "Expected sampled Azure DevOps user entitlements to include user identity data"
        );

        Ok(())
    }
}
