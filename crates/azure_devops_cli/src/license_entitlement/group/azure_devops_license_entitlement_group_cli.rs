use crate::license_entitlement::group::AzureDevOpsLicenseEntitlementGroupListArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementGroupArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsLicenseEntitlementGroupCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsLicenseEntitlementGroupCommand {
    /// List Azure DevOps group license entitlements.
    List(AzureDevOpsLicenseEntitlementGroupListArgs),
}

impl AzureDevOpsLicenseEntitlementGroupArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsLicenseEntitlementGroupCommand::List(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
