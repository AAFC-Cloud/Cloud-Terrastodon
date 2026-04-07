pub mod azure_policy_set_definition;
pub mod azure_policy_set_definition_browse;
pub mod azure_policy_set_definition_list;

pub use azure_policy_set_definition::AzurePolicySetDefinitionCommand;
pub use azure_policy_set_definition_browse::AzurePolicySetDefinitionBrowseArgs;
pub use azure_policy_set_definition_list::AzurePolicySetDefinitionListArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure policy set definitions.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicySetDefinitionArgs {
    #[command(subcommand)]
    pub command: AzurePolicySetDefinitionCommand,
}

impl AzurePolicySetDefinitionArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
