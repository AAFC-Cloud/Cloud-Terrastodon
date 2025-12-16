pub mod publisher;

use crate::cli::command::azure::vm::publisher::AzureVmPublisherArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// VM-related subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureVmCommand {
    /// Manage VM publishers and images.
    Publisher(AzureVmPublisherArgs),
}

/// Arguments for VM commands.
#[derive(Args, Debug, Clone)]
pub struct AzureVmArgs {
    #[command(subcommand)]
    pub command: AzureVmCommand,
}

impl AzureVmArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureVmCommand::Publisher(args) => args.invoke().await,
        }
    }
}
