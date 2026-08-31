use crate::cli::azure_devops::license_entitlement::group::AzureDevOpsLicenseEntitlementGroupListArgs;
use crate::cli::azure_devops::license_entitlement::user::AzureDevOpsLicenseEntitlementUserListArgs;
use crate::cli::azure_devops::license_entitlement::user::AzureDevOpsLicenseEntitlementUserRevokeArgs;
use crate::cli::azure_devops::license_entitlement::user::AzureDevOpsLicenseEntitlementUserShowArgs;
use crate::cli::azure_devops::license_entitlement::user::AzureDevOpsLicenseEntitlementUserSummaryArgs;
use crate::cli::azure_devops::license_entitlement::user::AzureDevOpsLicenseEntitlementUserUpdateArgs;
use crate::cli::azure_devops::license_entitlement::user::AzureDevOpsLicenseEntitlementUserUpdateTuiArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Azure DevOps license entitlement-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsLicenseEntitlementCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsLicenseEntitlementCommand {
    /// Operations on user license entitlements.
    User(AzureDevOpsLicenseEntitlementUserArgs),
    /// Operations on group license entitlements.
    Group(AzureDevOpsLicenseEntitlementGroupArgs),
}

#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsLicenseEntitlementUserCommand,
}

#[derive(facet::Facet, Debug, Clone)]
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

#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementGroupArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsLicenseEntitlementGroupCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsLicenseEntitlementGroupCommand {
    /// List Azure DevOps group license entitlements.
    List(AzureDevOpsLicenseEntitlementGroupListArgs),
}

impl AzureDevOpsLicenseEntitlementArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self.command {
            AzureDevOpsLicenseEntitlementCommand::User(args) => match args.command {
                AzureDevOpsLicenseEntitlementUserCommand::List(a) => a.invoke(auth_context).await?,
                AzureDevOpsLicenseEntitlementUserCommand::Summary(a) => {
                    a.invoke(auth_context).await?
                }
                AzureDevOpsLicenseEntitlementUserCommand::Update(a) => {
                    a.invoke(auth_context).await?
                }
                AzureDevOpsLicenseEntitlementUserCommand::UpdateTui(a) => {
                    a.invoke(auth_context).await?
                }
                AzureDevOpsLicenseEntitlementUserCommand::Show(a) => a.invoke(auth_context).await?,
                AzureDevOpsLicenseEntitlementUserCommand::Revoke(a) => {
                    a.invoke(auth_context).await?
                }
            },
            AzureDevOpsLicenseEntitlementCommand::Group(args) => match args.command {
                AzureDevOpsLicenseEntitlementGroupCommand::List(a) => a.invoke().await?,
            },
        }

        Ok(())
    }
}
