use crate::noninteractive::prelude::audit_azure_devops;
use clap::Args;
use eyre::Result;
use std::time::Duration;

/// Arguments for auditing Azure DevOps resources.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAuditArgs {
    /// The amount of time until an Azure DevOps paid license is considered unused.
    #[clap(long, default_value = "2months", value_parser = humantime::parse_duration)]
    paid_license_inactivity_threshold: Duration,
    /// The amount of time until an Azure DevOps "Test Plan" license is considered unused.
    /// This is more aggressive than the paid license threshold because test plan licenses are expensive.
    #[clap(long, default_value = "60days", value_parser = humantime::parse_duration)]
    test_license_inactivity_threshold: Duration,
}

impl AzureDevOpsAuditArgs {
    pub async fn invoke(self) -> Result<()> {
        audit_azure_devops(
            self.test_license_inactivity_threshold,
            self.paid_license_inactivity_threshold,
        )
        .await
    }
}
