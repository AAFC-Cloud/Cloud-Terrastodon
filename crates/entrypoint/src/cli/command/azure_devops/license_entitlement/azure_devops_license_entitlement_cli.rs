use crate::cli::azure_devops::license_entitlement::list::AzureDevOpsLicenseEntitlementListArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps license entitlement-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsLicenseEntitlementCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsLicenseEntitlementCommand {
    /// List Azure DevOps license entitlements.
    List(AzureDevOpsLicenseEntitlementListArgs),
}

impl AzureDevOpsLicenseEntitlementArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsLicenseEntitlementCommand::List(args) => args.invoke().await?,
        }

        Ok(())
    }
}
