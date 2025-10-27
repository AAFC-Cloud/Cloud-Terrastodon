use crate::noninteractive::prelude::audit_azure_devops;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps-specific commands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsCommand {
    /// Audit Azure DevOps resources for configuration issues.
    Audit,
}

impl AzureDevOpsCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureDevOpsCommand::Audit => {
                audit_azure_devops().await?;
            }
        }

        Ok(())
    }
}
