use clap::Args;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlement;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_azure_devops::update_azure_devops_user_license_entitlement;
use cloud_terrastodon_command::CacheInvalidatable;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use tracing::info;

#[derive(Args, Debug, Clone)]
/// Update an Azure DevOps user's license entitlement.
pub struct AzureDevOpsLicenseEntitlementUserUpdateArgs {
    #[arg(long)]
    pub user: AzureDevOpsUserArgument<'static>,

    /// The license that we expect the user to have
    #[arg(long)]
    pub has_license: Option<AzureDevOpsLicenseType>,

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

        if let Some(ref expected_license) = self.has_license {
            let entitlement = fetch_azure_devops_user_license_entitlement(&org_url, &self.user)
                .await
                .wrap_err("User does not have a license entitlement")?;

            if entitlement.license != *expected_license {
                bail!(
                    "User's current license {:?} does not match expected license {:?}",
                    entitlement.license,
                    expected_license
                );
            }
        }

        let _resp =
            update_azure_devops_user_license_entitlement(&org_url, &self.user, license.clone())
                .await?;

        info!(
            ?self.user,
            ?self.has_license,
            %self.license,
            "Attempted to update license entitlement for user"
        );

        fetch_azure_devops_user_license_entitlement(&org_url, self.user)
            .invalidate()
            .await?;

        Ok(())
    }
}
