use clap::Args;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_group_license_entitlements;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps group license entitlements for the organization.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementGroupListArgs {}

impl AzureDevOpsLicenseEntitlementGroupListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let entitlements = fetch_azure_devops_group_license_entitlements(&org_url).await?;
        to_writer_pretty(stdout(), &entitlements)?;
        Ok(())
    }
}
