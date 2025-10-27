pub mod azure_group_browse;
pub mod azure_group;
pub mod azure_group_list;

pub use azure_group_browse::AzureGroupBrowseArgs;
use clap::Args;
use eyre::Result;
pub use azure_group::AzureGroupCommand;
pub use azure_group_list::AzureGroupListArgs;

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
