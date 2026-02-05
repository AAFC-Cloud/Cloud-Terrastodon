use crate::cli::azure_devops::license_entitlement::user::AzureDevOpsLicenseEntitlementUserMatcher;
use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_azure_devops::prelude::update_azure_devops_user_license_entitlement;
use eyre::Result;
use eyre::bail;
use tracing::info;

#[derive(Args, Debug, Clone)]
/// Update an Azure DevOps user's license entitlement.
pub struct AzureDevOpsLicenseEntitlementUserUpdateArgs {
    #[clap(flatten)]
    pub user_matcher: AzureDevOpsLicenseEntitlementUserMatcher,

    /// Desired license kind (e.g. "Account-Express", "Account-Advanced"). Required unless using `tui` subcommand.
    #[arg(long)]
    pub license: String,
}

impl AzureDevOpsLicenseEntitlementUserUpdateArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;

        let license = self.license.parse::<AzureDevOpsLicenseType>()?;
        if let AzureDevOpsLicenseType::Other(s) = &license {
            bail!("Invalid license kind specified: {}", s);
        };

        let resp = update_azure_devops_user_license_entitlement(
            &org_url,
            self.user_matcher.as_argument()?,
            license.clone(),
        )
        .await?;

        info!(
            ?self.user_matcher,
            %self.license,
            ?resp,
            "Successfully updated license entitlement for user"
        );

        Ok(())
    }
}
