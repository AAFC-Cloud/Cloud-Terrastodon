use cloud_terrastodon_credentials::AuthContext;
pub mod azure_policy_set_definition_browse_cli;
pub mod azure_policy_set_definition_cli;
pub mod azure_policy_set_definition_list_cli;
pub mod azure_policy_set_definition_show_cli;

pub use azure_policy_set_definition_browse_cli::AzurePolicySetDefinitionBrowseArgs;
pub use azure_policy_set_definition_cli::AzurePolicySetDefinitionCommand;
pub use azure_policy_set_definition_list_cli::AzurePolicySetDefinitionListArgs;
pub use azure_policy_set_definition_show_cli::AzurePolicySetDefinitionShowArgs;
use eyre::Result;

/// Manage Azure policy set definitions.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePolicySetDefinitionArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePolicySetDefinitionCommand,
}

impl AzurePolicySetDefinitionArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
