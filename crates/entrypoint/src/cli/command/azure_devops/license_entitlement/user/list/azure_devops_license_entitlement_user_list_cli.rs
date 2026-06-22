use clap::Args;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps license entitlements (users) for the organization.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserListArgs {}

impl AzureDevOpsLicenseEntitlementUserListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let entitlements = fetch_azure_devops_user_license_entitlements(&org_url).await?;
        to_writer_pretty(stdout(), &entitlements)?;
        Ok(())
    }
}
