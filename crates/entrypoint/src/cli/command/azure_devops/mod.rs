pub mod azure_devops_command;

pub use azure_devops_command::AzureDevOpsCommand;
use clap::Args;
use eyre::Result;

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
