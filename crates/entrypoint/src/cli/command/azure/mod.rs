pub mod azure_command;
pub mod group;
pub mod policy;
pub mod pim;
pub mod tag;

use clap::Args;
use eyre::Result;

use crate::cli::azure::azure_command::AzureCommand;

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
