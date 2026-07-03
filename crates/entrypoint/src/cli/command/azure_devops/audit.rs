use crate::cli::scalar_args::HumantimeDurationCli;
use crate::noninteractive::audit_azure_devops;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use eyre::Result;

/// Arguments for auditing Azure DevOps resources.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsAuditArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// The amount of time until an Azure DevOps paid license is considered unused.
    #[facet(figue::named, default = HumantimeDurationCli("2months".parse().unwrap()))]
    paid_license_inactivity_threshold: HumantimeDurationCli,
    /// The amount of time until an Azure DevOps "Test Plan" license is considered unused.
    /// This is more aggressive than the paid license threshold because test plan licenses are expensive.
    #[facet(figue::named, default = HumantimeDurationCli("60days".parse().unwrap()))]
    test_license_inactivity_threshold: HumantimeDurationCli,
}

impl AzureDevOpsAuditArgs {
    pub async fn invoke(self) -> Result<()> {
        audit_azure_devops(
            self.tenant.resolve().await?,
            self.test_license_inactivity_threshold.0.into(),
            self.paid_license_inactivity_threshold.0.into(),
        )
        .await
    }
}
