use super::audit::AzureAuditArgs;
use super::find::AzureFindArgs;
use super::pim::AzurePimArgs;
use super::policy::AzurePolicyArgs;
use super::resource::AzureResourceArgs;
use super::resource_group::AzureResourceGroupArgs;
use super::role::AzureRoleArgs;
use super::subscription::AzureSubscriptionArgs;
use super::tag::AzureTagArgs;
use super::tenant::AzureTenantArgs;
use super::vm::AzureVmArgs;
use crate::cli::azure::entra::AzureEntraArgs;
use crate::cli::azure_devops::AzureDevOpsArgs;
use clap::Subcommand;
use eyre::Result;

/// Azure-specific commands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureCommand {
    /// Audit Azure resources for configuration issues.
    Audit(AzureAuditArgs),
    /// Find resources where resource JSON contains the given text.
    Find(AzureFindArgs),
    /// Manage Azure resource groups.
    #[command(aliases = ["rg", "group"])]
    ResourceGroup(AzureResourceGroupArgs),
    /// Manage Azure policy resources.
    Policy(AzurePolicyArgs),
    /// Manage Azure resource tags.
    Tag(AzureTagArgs),
    /// Manage Azure resources.
    #[command(alias = "res")]
    Resource(AzureResourceArgs),
    /// Manage Azure role-based access control.
    Role(AzureRoleArgs),
    /// Manage Azure Privileged Identity Management operations.
    Pim(AzurePimArgs),
    /// Entra (Azure AD) commands.
    #[command(alias = "ad")]
    Entra(AzureEntraArgs),
    /// VM-related commands (images, publishers, sizes, etc.)
    Vm(AzureVmArgs),
    /// Manage subscriptions within the tenant.
    #[command(alias = "sub")]
    Subscription(AzureSubscriptionArgs),
    /// Manage tracked tenants for later login flows.
    Tenant(AzureTenantArgs),
    /// Azure DevOps-specific commands.
    #[command(alias = "devops")]
    DevOps(AzureDevOpsArgs),
}

impl AzureCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureCommand::Audit(args) => {
                args.invoke().await?;
            }
            AzureCommand::Find(args) => {
                args.invoke().await?;
            }
            AzureCommand::ResourceGroup(args) => {
                args.invoke().await?;
            }
            AzureCommand::Policy(args) => {
                args.invoke().await?;
            }
            AzureCommand::Tag(args) => {
                args.invoke().await?;
            }
            AzureCommand::Resource(args) => {
                args.invoke().await?;
            }
            AzureCommand::Role(args) => {
                args.invoke().await?;
            }
            AzureCommand::Pim(args) => {
                args.invoke().await?;
            }
            AzureCommand::Entra(args) => {
                args.invoke().await?;
            }
            AzureCommand::Vm(args) => {
                args.invoke().await?;
            }
            AzureCommand::Subscription(args) => {
                args.invoke().await?;
            }
            AzureCommand::Tenant(args) => {
                args.invoke().await?;
            }
            AzureCommand::DevOps(args) => {
                args.invoke().await?;
            }
        }

        Ok(())
    }
}
