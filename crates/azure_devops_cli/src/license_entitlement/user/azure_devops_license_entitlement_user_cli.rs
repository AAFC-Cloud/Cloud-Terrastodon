use crate::license_entitlement::user::AzureDevOpsLicenseEntitlementUserListArgs;
use crate::license_entitlement::user::AzureDevOpsLicenseEntitlementUserRevokeArgs;
use crate::license_entitlement::user::AzureDevOpsLicenseEntitlementUserShowArgs;
use crate::license_entitlement::user::AzureDevOpsLicenseEntitlementUserSummaryArgs;
use crate::license_entitlement::user::AzureDevOpsLicenseEntitlementUserUpdateArgs;
use crate::license_entitlement::user::AzureDevOpsLicenseEntitlementUserUpdateTuiArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsLicenseEntitlementUserCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsLicenseEntitlementUserCommand {
    /// List Azure DevOps user license entitlements.
    List(AzureDevOpsLicenseEntitlementUserListArgs),
    /// Summarize Azure DevOps user license entitlements by license type.
    Summary(AzureDevOpsLicenseEntitlementUserSummaryArgs),
    /// Update Azure DevOps user license entitlement.
    Update(AzureDevOpsLicenseEntitlementUserUpdateArgs),
    /// Update Azure DevOps user license entitlement via TUI picker.
    UpdateTui(AzureDevOpsLicenseEntitlementUserUpdateTuiArgs),
    /// Show a specific user's license entitlement by id.
    Show(AzureDevOpsLicenseEntitlementUserShowArgs),
    /// Revoke group rule assignment (show groups that grant this user's license).
    Revoke(AzureDevOpsLicenseEntitlementUserRevokeArgs),
}

impl AzureDevOpsLicenseEntitlementUserArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsLicenseEntitlementUserCommand::List(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureDevOpsLicenseEntitlementUserCommand::Summary(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureDevOpsLicenseEntitlementUserCommand::Update(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureDevOpsLicenseEntitlementUserCommand::UpdateTui(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureDevOpsLicenseEntitlementUserCommand::Show(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureDevOpsLicenseEntitlementUserCommand::Revoke(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
