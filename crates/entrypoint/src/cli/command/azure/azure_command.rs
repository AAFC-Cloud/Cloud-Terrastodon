use super::audit::AzureAuditArgs;
use super::group::AzureGroupArgs;
use super::pim::AzurePimArgs;
use super::policy::AzurePolicyArgs;
use super::role::AzureRoleArgs;
use super::tag::AzureTagArgs;
use super::vm::AzureVmArgs;
use clap::Subcommand;
use eyre::Result;

/// Azure-specific commands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureCommand {
    /// Audit Azure resources for configuration issues.
    Audit(AzureAuditArgs),
    /// Manage Azure resource groups.
    Group(AzureGroupArgs),
    /// Manage Azure policy resources.
    Policy(AzurePolicyArgs),
    /// Manage Azure resource tags.
    Tag(AzureTagArgs),
    /// Manage Azure role-based access control.
    Role(AzureRoleArgs),
    /// Manage Azure Privileged Identity Management operations.
    Pim(AzurePimArgs),
    /// VM-related commands (images, publishers, sizes, etc.)
    Vm(AzureVmArgs),
}

impl AzureCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureCommand::Audit(args) => {
                args.invoke().await?;
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
            AzureCommand::Role(args) => {
                args.invoke().await?;
            }
            AzureCommand::Pim(args) => {
                args.invoke().await?;
            }
            AzureCommand::Vm(args) => {
                args.invoke().await?;
            }
        }

        Ok(())
    }
}
