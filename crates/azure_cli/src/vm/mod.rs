use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod browse;
pub mod publisher;

use crate::vm::browse::AzureVmBrowseArgs;
use crate::vm::publisher::AzureVmPublisherArgs;
use cloud_terrastodon_auth::AuthContext;
use eyre::Result;

/// Virtual machine related subcommands.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureVmCommand {
    /// Manage virtual machine publishers and images.
    Publisher(AzureVmPublisherArgs),
    /// Browse virtual machine related areas
    Browse(AzureVmBrowseArgs),
}

/// Arguments for VM commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureVmArgs {
    #[facet(figue::subcommand)]
    pub command: AzureVmCommand,
}

impl AzureVmArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureVmCommand::Publisher(args) => args.invoke(auth_context, executor).await,
            AzureVmCommand::Browse(args) => args.invoke(auth_context, executor).await,
        }
    }
}
