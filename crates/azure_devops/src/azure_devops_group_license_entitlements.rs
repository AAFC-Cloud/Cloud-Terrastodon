use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsGroupLicenseEntitlement;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

pub struct AzureDevOpsGroupLicenseEntitlementsRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
}

pub fn fetch_azure_devops_group_license_entitlements<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
) -> AzureDevOpsGroupLicenseEntitlementsRequest<'a> {
    AzureDevOpsGroupLicenseEntitlementsRequest { org_url }
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
            self.org_url.organization_name.as_ref(),
            "license",
            "entitlement",
            "list-groups",
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching Azure DevOps group entitlements");
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["devops", "invoke"]);
        let org = self.org_url.to_string();
        cmd.args(["--organization", org.as_str()]);
        cmd.args(["--area", "MemberEntitlementManagement"]);
        cmd.args(["--resource", "GroupEntitlements"]);
        cmd.args(["--api-version", "7.2-preview"]);
        cmd.args(["--encoding", "utf-8"]);
        cmd.cache(self.cache_key());

        #[derive(Deserialize)]
        struct InvokeResponse {
            continuation_token: Option<Value>,
            count: u32,
            value: Vec<AzureDevOpsGroupLicenseEntitlement>,
        }

        let resp = cmd.run::<InvokeResponse>().await?;
        let entitlements = resp.value;

        debug!("Found {} Azure DevOps group entitlements", resp.count);

        if resp.continuation_token.is_some() {
            todo!("Add support for continuation token...");
        }

        Ok(entitlements)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsGroupLicenseEntitlementsRequest<'a>, 'a);

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let entitlements = fetch_azure_devops_group_license_entitlements(&org_url).await?;
        println!("Found {} group license entitlements", entitlements.len());
        for entitlement in entitlements.iter().take(5) {
            println!(
                "Group {} ({}) has {}",
                entitlement.group.display_name,
                entitlement.group.origin_id,
                entitlement.license_rule.account_license_type
            );
        }

        Ok(())
    }
}
