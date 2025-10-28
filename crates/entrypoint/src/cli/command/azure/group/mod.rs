pub mod azure_group;
pub mod azure_group_browse;
pub mod azure_group_list;

pub use azure_group::AzureGroupCommand;
pub use azure_group_browse::AzureGroupBrowseArgs;
pub use azure_group_list::AzureGroupListArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure resource groups.
#[derive(Args, Debug, Clone)]
pub struct AzureGroupArgs {
    #[command(subcommand)]
    pub command: AzureGroupCommand,
}

impl AzureGroupArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
