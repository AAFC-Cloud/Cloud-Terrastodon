use cloud_terrastodon_credentials::AuthContext;
pub mod azure_policy_definition_browse_cli;
pub mod azure_policy_definition_cli;
pub mod azure_policy_definition_list_cli;
pub mod azure_policy_definition_show_cli;

pub use azure_policy_definition_browse_cli::AzurePolicyDefinitionBrowseArgs;
pub use azure_policy_definition_cli::AzurePolicyDefinitionCommand;
pub use azure_policy_definition_list_cli::AzurePolicyDefinitionListArgs;
pub use azure_policy_definition_show_cli::AzurePolicyDefinitionShowArgs;
use eyre::Result;

/// Manage Azure policy definitions.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePolicyDefinitionArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePolicyDefinitionCommand,
}

impl AzurePolicyDefinitionArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
