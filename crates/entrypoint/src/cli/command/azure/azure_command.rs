use super::group::AzureGroupArgs;
use super::policy::AzurePolicyArgs;
use super::pim::AzurePimArgs;
use super::tag::AzureTagArgs;
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
    /// Manage Azure resource tags.
    Tag(AzureTagArgs),
    /// Manage Azure Privileged Identity Management operations.
    Pim(AzurePimArgs),
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
            AzureCommand::Tag(args) => {
                args.invoke().await?;
            }
            AzureCommand::Pim(args) => {
                args.invoke().await?;
            }
        }

        Ok(())
    }
}
