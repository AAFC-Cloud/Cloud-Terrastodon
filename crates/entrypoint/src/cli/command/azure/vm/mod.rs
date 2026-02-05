pub mod browse;
pub mod publisher;

use crate::cli::azure::vm::browse::AzureVmBrowseArgs;
use crate::cli::command::azure::vm::publisher::AzureVmPublisherArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Virtual machine related subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureVmCommand {
    /// Manage virtual machine publishers and images.
    Publisher(AzureVmPublisherArgs),
    /// Browse virtual machine related areas
    Browse(AzureVmBrowseArgs),
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
            AzureVmCommand::Browse(args) => args.invoke().await,
        }
    }
}
