use super::group::AzureGroupArgs;
use super::policy::AzurePolicyArgs;
use crate::noninteractive::prelude::audit_azure;
use clap::Subcommand;
use eyre::Result;

/// Azure-specific commands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureCommand {
    /// Audit Azure resources for configuration issues.
    Audit,
    /// Manage Azure resource groups.
    Group(AzureGroupArgs),
    /// Manage Azure policy resources.
    Policy(AzurePolicyArgs),
}

impl AzureCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureCommand::Audit => {
                audit_azure().await?;
            }
            AzureCommand::Group(args) => {
                args.invoke().await?;
            }
            AzureCommand::Policy(args) => {
                args.invoke().await?;
            }
        }

        Ok(())
    }
}
