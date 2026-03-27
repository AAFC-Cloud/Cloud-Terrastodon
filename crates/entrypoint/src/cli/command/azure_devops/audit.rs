use crate::noninteractive::audit_azure_devops;
use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use eyre::Result;
use std::time::Duration;

/// Arguments for auditing Azure DevOps resources.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAuditArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

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
            self.tenant.resolve().await?,
            self.test_license_inactivity_threshold,
            self.paid_license_inactivity_threshold,
        )
        .await
    }
}
