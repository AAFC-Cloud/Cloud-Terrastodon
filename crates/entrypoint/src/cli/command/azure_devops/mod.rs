pub mod azure_devops_command;

use clap::Args;
use eyre::Result;

use crate::cli::azure_devops::azure_devops_command::AzureDevOpsCommand;

/// Arguments for Azure DevOps-specific operations.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsCommand,
}

impl AzureDevOpsArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
