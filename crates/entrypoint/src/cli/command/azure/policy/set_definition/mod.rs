pub mod azure_policy_set_definition;
pub mod azure_policy_set_definition_browse;
pub mod azure_policy_set_definition_list;
pub mod azure_policy_set_definition_show;

pub use azure_policy_set_definition::AzurePolicySetDefinitionCommand;
pub use azure_policy_set_definition_browse::AzurePolicySetDefinitionBrowseArgs;
pub use azure_policy_set_definition_list::AzurePolicySetDefinitionListArgs;
pub use azure_policy_set_definition_show::AzurePolicySetDefinitionShowArgs;
use eyre::Result;

/// Manage Azure policy set definitions.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePolicySetDefinitionArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePolicySetDefinitionCommand,
}

impl AzurePolicySetDefinitionArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
