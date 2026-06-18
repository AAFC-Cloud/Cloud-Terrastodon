use clap::Args;
use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlement;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// Show a single Azure DevOps user license entitlement by user id.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserShowArgs {
    pub user: AzureDevOpsUserArgument<'static>,
}

impl AzureDevOpsLicenseEntitlementUserShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let found =
            fetch_azure_devops_user_license_entitlement(&org_url, &self.user).await?;
        to_writer_pretty(stdout(), &found)?;
        Ok(())
    }
}
