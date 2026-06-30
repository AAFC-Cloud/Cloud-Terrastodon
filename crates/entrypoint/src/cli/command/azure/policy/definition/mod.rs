pub mod azure_policy_definition;
pub mod azure_policy_definition_browse;
pub mod azure_policy_definition_list;
pub mod azure_policy_definition_show;

pub use azure_policy_definition::AzurePolicyDefinitionCommand;
pub use azure_policy_definition_browse::AzurePolicyDefinitionBrowseArgs;
pub use azure_policy_definition_list::AzurePolicyDefinitionListArgs;
pub use azure_policy_definition_show::AzurePolicyDefinitionShowArgs;
use eyre::Result;

/// Manage Azure policy definitions.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePolicyDefinitionArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePolicyDefinitionCommand,
}

impl AzurePolicyDefinitionArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
