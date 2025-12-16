pub mod azure_vm_publisher_list;

pub use azure_vm_publisher_list::AzureVmPublisherListArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;


#[derive(Args, Debug, Clone)]
pub struct AzureVmPublisherArgs {
    #[command(subcommand)]
    pub command: AzureVmPublisherCommand,
}

impl AzureVmPublisherArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

/// Manage VM publishers.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureVmPublisherCommand {
    /// List VM publishers for a subscription and location.
    List(AzureVmPublisherListArgs),
}

impl AzureVmPublisherCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureVmPublisherCommand::List(args) => args.invoke().await,
        }
    }
}