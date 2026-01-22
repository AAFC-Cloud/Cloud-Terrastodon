use clap::Args;
use clap::Subcommand;
use eyre::Result;

use crate::cli::azure_devops::license_entitlement::user::{
    AzureDevOpsLicenseEntitlementUserListArgs,
    AzureDevOpsLicenseEntitlementUserUpdateArgs,
    AzureDevOpsLicenseEntitlementUserUpdateTuiArgs,
};
use crate::cli::azure_devops::license_entitlement::group::AzureDevOpsLicenseEntitlementGroupListArgs;

/// Azure DevOps license entitlement-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsLicenseEntitlementCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsLicenseEntitlementCommand {
    /// Operations on user license entitlements.
    User(AzureDevOpsLicenseEntitlementUserArgs),
    /// Operations on group license entitlements.
    Group(AzureDevOpsLicenseEntitlementGroupArgs),
}

#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsLicenseEntitlementUserCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsLicenseEntitlementUserCommand {
    /// List Azure DevOps user license entitlements.
    List(AzureDevOpsLicenseEntitlementUserListArgs),
    /// Update Azure DevOps user license entitlement.
    Update(AzureDevOpsLicenseEntitlementUserUpdateArgs),
    /// Update Azure DevOps user license entitlement via TUI picker.
    UpdateTui(AzureDevOpsLicenseEntitlementUserUpdateTuiArgs),
}

#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementGroupArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsLicenseEntitlementGroupCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsLicenseEntitlementGroupCommand {
    /// List Azure DevOps group license entitlements.
    List(AzureDevOpsLicenseEntitlementGroupListArgs),
}

impl AzureDevOpsLicenseEntitlementArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsLicenseEntitlementCommand::User(args) => match args.command {
                AzureDevOpsLicenseEntitlementUserCommand::List(a) => a.invoke().await?,
                AzureDevOpsLicenseEntitlementUserCommand::Update(a) => a.invoke().await?,
                AzureDevOpsLicenseEntitlementUserCommand::UpdateTui(a) => a.invoke().await?,
            },
            AzureDevOpsLicenseEntitlementCommand::Group(args) => match args.command {
                AzureDevOpsLicenseEntitlementGroupCommand::List(a) => a.invoke().await?,
            },
        }

        Ok(())
    }
}
