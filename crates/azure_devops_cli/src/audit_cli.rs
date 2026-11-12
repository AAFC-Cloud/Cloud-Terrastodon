use crate::audit_azure_devops;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cli_types::HumantimeDurationCli;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;
use std::time::Duration;

/// Arguments for auditing Azure DevOps resources.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsAuditArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,

    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// The amount of time until an Azure DevOps paid license is considered unused.
    /// Defaults to two humantime months (30.44 days each).
    #[facet(figue::named, default = HumantimeDurationCli(Duration::from_secs(2 * 2_630_016).into()))]
    paid_license_inactivity_threshold: HumantimeDurationCli,
    /// The amount of time until an Azure DevOps "Test Plan" license is considered unused.
    /// This is more aggressive than the paid license threshold because test plan licenses are expensive.
    #[facet(figue::named, default = HumantimeDurationCli(Duration::from_secs(60 * 24 * 60 * 60).into()))]
    test_license_inactivity_threshold: HumantimeDurationCli,
}

impl AzureDevOpsAuditArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        audit_azure_devops(
            self.org,
            self.test_license_inactivity_threshold.0.into(),
            self.paid_license_inactivity_threshold.0.into(),
            authentication,
            executor,
        )
        .await
    }
}
