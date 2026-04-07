use super::AzurePolicySetDefinitionBrowseArgs;
use super::AzurePolicySetDefinitionListArgs;
use super::AzurePolicySetDefinitionShowArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for managing Azure policy set definitions.
#[derive(Subcommand, Debug, Clone)]
pub enum AzurePolicySetDefinitionCommand {
    /// List all Azure policy set definitions accessible to the account.
    List(AzurePolicySetDefinitionListArgs),
    /// Browse Azure policy set definitions in an interactive manner.
    Browse(AzurePolicySetDefinitionBrowseArgs),
    /// Show a single Azure policy set definition by id, name, or display name.
    Show(AzurePolicySetDefinitionShowArgs),
}

impl AzurePolicySetDefinitionCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzurePolicySetDefinitionCommand::List(args) => args.invoke().await,
            AzurePolicySetDefinitionCommand::Browse(args) => args.invoke().await,
            AzurePolicySetDefinitionCommand::Show(args) => args.invoke().await,
        }
    }
}
