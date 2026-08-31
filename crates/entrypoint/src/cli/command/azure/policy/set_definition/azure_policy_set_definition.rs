use super::AzurePolicySetDefinitionBrowseArgs;
use super::AzurePolicySetDefinitionListArgs;
use super::AzurePolicySetDefinitionShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Subcommands for managing Azure policy set definitions.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzurePolicySetDefinitionCommand {
    /// List all Azure policy set definitions accessible to the account.
    List(AzurePolicySetDefinitionListArgs),
    /// Browse Azure policy set definitions in an interactive manner.
    Browse(AzurePolicySetDefinitionBrowseArgs),
    /// Show a single Azure policy set definition by id, name, or display name.
    Show(AzurePolicySetDefinitionShowArgs),
}

impl AzurePolicySetDefinitionCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzurePolicySetDefinitionCommand::List(args) => args.invoke(auth_context).await,
            AzurePolicySetDefinitionCommand::Browse(args) => args.invoke(auth_context).await,
            AzurePolicySetDefinitionCommand::Show(args) => args.invoke(auth_context).await,
        }
    }
}
