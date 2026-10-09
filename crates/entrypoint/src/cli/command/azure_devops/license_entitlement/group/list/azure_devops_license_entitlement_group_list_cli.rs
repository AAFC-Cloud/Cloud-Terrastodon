use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::fetch_azure_devops_group_license_entitlements;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps group license entitlements for the organization.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementGroupListArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsLicenseEntitlementGroupListArgs {
    pub async fn invoke(self) -> Result<()> {
        let entitlements = fetch_azure_devops_group_license_entitlements(&self.org).await?;
        to_writer_pretty(stdout(), &entitlements)?;
        Ok(())
    }
}
