pub mod browse;
pub mod publisher;

use crate::cli::azure::vm::browse::AzureVmBrowseArgs;
use crate::cli::command::azure::vm::publisher::AzureVmPublisherArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Virtual machine related subcommands.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureVmCommand {
    /// Manage virtual machine publishers and images.
    Publisher(AzureVmPublisherArgs),
    /// Browse virtual machine related areas
    Browse(AzureVmBrowseArgs),
}

/// Arguments for VM commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureVmArgs {
    #[facet(figue::subcommand)]
    pub command: AzureVmCommand,
}

impl AzureVmArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self.command {
            AzureVmCommand::Publisher(args) => args.invoke(auth_context).await,
            AzureVmCommand::Browse(args) => args.invoke(auth_context).await,
        }
    }
}
