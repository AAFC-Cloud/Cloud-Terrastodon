pub mod audit;
pub mod azure_command;
pub mod entra;
pub mod group;
pub mod pim;
pub mod policy;
pub mod role;
pub mod subscription;
pub mod tag;
pub mod vm;

use crate::cli::azure::azure_command::AzureCommand;
use clap::Args;
use eyre::Result;

/// Arguments for Azure-specific operations.
#[derive(Args, Debug, Clone)]
pub struct AzureArgs {
    #[command(subcommand)]
    pub command: AzureCommand,
}

impl AzureArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
