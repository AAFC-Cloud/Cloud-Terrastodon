use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_rest::MicrosoftContinuationToken;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsUserLicenseEntitlementListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

pub fn fetch_azure_devops_user_license_entitlements<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    auth_context: &'a AzureDevOpsAuthContext,
) -> AzureDevOpsUserLicenseEntitlementListRequest<'a> {
    AzureDevOpsUserLicenseEntitlementListRequest {
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsUserLicenseEntitlementListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
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
            self.org_url.name(),
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
        let mut continuation: Option<MicrosoftContinuationToken> = None;
        let mut count = 0;
        let cache_key = self.cache_key();
        let mut page_index = 0;
        loop {
            let mut query = vec![("api-version", "7.1-preview.3")];
            if let Some(token) = continuation.as_ref() {
                query.push(("continuationToken", token.as_str()));
            }
            let url =
                self.org_url
                    .api_url("vsaex.dev.azure.com", "_apis/userentitlements", &query)?;
            let request = RestRequest::from_method_and_url(Method::GET, url)?
                .cache(cache_key.join(page_index.to_string()))
                .azure_devops_auth_context(self.auth_context.as_ref())?;
            let (response, next_continuation) = request
                .receive_with_ms_continuation_token::<InvokeResponse>()
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
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let auth_context = AuthContext::explicit_azure_cli();
        let azure_devops_auth_context = AzureDevOpsAuthContext::new(&auth_context)?;
        let entitlements =
            fetch_azure_devops_user_license_entitlements(&org_url, &azure_devops_auth_context)
                .await?;
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
