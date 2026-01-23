use clap::Args;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use eyre::bail;
use serde_json::to_writer_pretty;
use std::io::stdout;

use crate::cli::azure_devops::license_entitlement::user::AzureDevOpsLicenseEntitlementUserMatcher;

/// Show a single Azure DevOps user license entitlement by user id.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserShowArgs {
    #[clap(flatten)]
    pub user_matcher: AzureDevOpsLicenseEntitlementUserMatcher
}

impl AzureDevOpsLicenseEntitlementUserShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let user_predicate = self.user_matcher.as_predicate()?;
        let org_url = get_default_organization_url().await?;
        let entitlements = fetch_azure_devops_user_license_entitlements(&org_url).await?;

        match entitlements.into_iter().find(|e| user_predicate(e)) {
            Some(ent) => {
                to_writer_pretty(stdout(), &ent)?;
                Ok(())
            }
            None => bail!("No license entitlement found for user matching criteria {:?}", self.user_matcher),
        }
    }
}
