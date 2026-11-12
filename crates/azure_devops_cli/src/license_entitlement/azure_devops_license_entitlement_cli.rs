use crate::license_entitlement::group::AzureDevOpsLicenseEntitlementGroupArgs;
use crate::license_entitlement::user::AzureDevOpsLicenseEntitlementUserArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps license entitlement-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsLicenseEntitlementCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsLicenseEntitlementCommand {
    /// Operations on user license entitlements.
    User(AzureDevOpsLicenseEntitlementUserArgs),
    /// Operations on group license entitlements.
    Group(AzureDevOpsLicenseEntitlementGroupArgs),
}

impl AzureDevOpsLicenseEntitlementArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsLicenseEntitlementCommand::User(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureDevOpsLicenseEntitlementCommand::Group(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
