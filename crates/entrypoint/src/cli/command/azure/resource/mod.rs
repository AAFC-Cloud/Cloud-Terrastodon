pub mod azure_resource;
pub mod azure_resource_browse;
pub mod azure_resource_list;
pub mod azure_resource_show;

pub use azure_resource::AzureResourceCommand;
pub use azure_resource_browse::AzureResourceBrowseArgs;
pub use azure_resource_list::AzureResourceListArgs;
pub use azure_resource_show::AzureResourceShowArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure resources.
#[derive(Args, Debug, Clone)]
pub struct AzureResourceArgs {
    #[command(subcommand)]
    pub command: AzureResourceCommand,
}

impl AzureResourceArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
