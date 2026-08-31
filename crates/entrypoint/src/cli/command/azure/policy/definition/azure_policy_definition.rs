use cloud_terrastodon_credentials::AuthContext;
use super::AzurePolicyDefinitionBrowseArgs;
use super::AzurePolicyDefinitionListArgs;
use super::AzurePolicyDefinitionShowArgs;
use eyre::Result;

/// Subcommands for managing Azure policy definitions.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzurePolicyDefinitionCommand {
    /// List all Azure policy definitions accessible to the account.
    List(AzurePolicyDefinitionListArgs),
    /// Browse Azure policy definitions in an interactive manner.
    Browse(AzurePolicyDefinitionBrowseArgs),
    /// Show a single Azure policy definition by id, name, or display name.
    Show(AzurePolicyDefinitionShowArgs),
}

impl AzurePolicyDefinitionCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
    ) -> Result<()> {
        match self {
            AzurePolicyDefinitionCommand::List(args) => args.invoke(auth_context).await,
            AzurePolicyDefinitionCommand::Browse(args) => args.invoke(auth_context).await,
            AzurePolicyDefinitionCommand::Show(args) => args.invoke(auth_context).await,
        }
    }
}
