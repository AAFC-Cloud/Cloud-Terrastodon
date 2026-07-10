use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use facet_json::RawJson;
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

        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["devops", "invoke"]);
        let org = self.org_url.to_string();
        cmd.args(["--organization", org.as_str()]);
        cmd.args(["--area", "licensing"]);
        cmd.args(["--resource", "entitlements"]);
        cmd.args(["--api-version", "7.2-preview"]);
        cmd.args(["--encoding", "utf-8"]);
        cmd.cache(self.cache_key());

        #[derive(facet::Facet)]
        struct InvokeResponse {
            continuation_token: Option<RawJson<'static>>,
            count: u32,
            value: Vec<AzureDevOpsUserLicenseEntitlement>,
        }

        let resp = cmd.run::<InvokeResponse>().await?;
        let entitlements = resp.value;

        debug!("Found {} Azure DevOps user entitlements", resp.count);

        if resp.continuation_token.is_some() {
            todo!("Add support for continuation token...");
        }

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
use arbitrary::Arbitrary;
