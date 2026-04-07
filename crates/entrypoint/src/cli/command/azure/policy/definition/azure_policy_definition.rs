use super::AzurePolicyDefinitionBrowseArgs;
use super::AzurePolicyDefinitionListArgs;
use super::AzurePolicyDefinitionShowArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for managing Azure policy definitions.
#[derive(Subcommand, Debug, Clone)]
pub enum AzurePolicyDefinitionCommand {
    /// List all Azure policy definitions accessible to the account.
    List(AzurePolicyDefinitionListArgs),
    /// Browse Azure policy definitions in an interactive manner.
    Browse(AzurePolicyDefinitionBrowseArgs),
    /// Show a single Azure policy definition by id, name, or display name.
    Show(AzurePolicyDefinitionShowArgs),
}

impl AzurePolicyDefinitionCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzurePolicyDefinitionCommand::List(args) => args.invoke().await,
            AzurePolicyDefinitionCommand::Browse(args) => args.invoke().await,
            AzurePolicyDefinitionCommand::Show(args) => args.invoke().await,
        }
    }
}
