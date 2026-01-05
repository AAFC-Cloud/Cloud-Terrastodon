use super::audit::AzureDevOpsAuditArgs;
use super::azure_devops_rest_command::AzureDevOpsRestArgs;
use super::project::AzureDevOpsProjectArgs;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps-specific commands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsCommand {
    /// Audit Azure DevOps resources for configuration issues.
    Audit(AzureDevOpsAuditArgs),
    /// Issue raw Azure DevOps REST requests.
    Rest(AzureDevOpsRestArgs),
    /// Project-level operations (list, show, ...)
    Project(AzureDevOpsProjectArgs),
}

impl AzureDevOpsCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureDevOpsCommand::Audit(args) => {
                args.invoke().await?;
            }
            AzureDevOpsCommand::Rest(args) => {
                args.invoke().await?;
            }
            AzureDevOpsCommand::Project(args) => {
                args.invoke().await?;
            }
        }

        Ok(())
    }
}
