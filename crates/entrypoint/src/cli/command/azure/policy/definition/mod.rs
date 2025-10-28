pub mod azure_policy_definition;
pub mod azure_policy_definition_browse;
pub mod azure_policy_definition_list;

pub use azure_policy_definition::AzurePolicyDefinitionCommand;
pub use azure_policy_definition_browse::AzurePolicyDefinitionBrowseArgs;
pub use azure_policy_definition_list::AzurePolicyDefinitionListArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure policy definitions.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicyDefinitionArgs {
    #[command(subcommand)]
    pub command: AzurePolicyDefinitionCommand,
}

impl AzurePolicyDefinitionArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
