pub mod azure_command;
pub mod group;
pub mod policy;

pub use azure_command::AzureCommand;
use clap::Args;
use eyre::Result;
pub use group::AzureGroupArgs;
pub use group::AzureGroupBrowseArgs;
pub use group::AzureGroupCommand;
pub use group::AzureGroupListArgs;
pub use policy::AzurePolicyArgs;
pub use policy::AzurePolicyCommand;
pub use policy::AzurePolicyDefinitionArgs;
pub use policy::AzurePolicyDefinitionBrowseArgs;
pub use policy::AzurePolicyDefinitionCommand;
pub use policy::AzurePolicyDefinitionListArgs;

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
