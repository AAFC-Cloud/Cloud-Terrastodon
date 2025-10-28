pub mod definition;

use clap::Args;
use clap::Subcommand;
pub use definition::AzurePolicyDefinitionArgs;
pub use definition::AzurePolicyDefinitionBrowseArgs;
pub use definition::AzurePolicyDefinitionCommand;
pub use definition::AzurePolicyDefinitionListArgs;
use eyre::Result;

/// Manage Azure policy resources.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicyArgs {
    #[command(subcommand)]
    pub command: AzurePolicyCommand,
}

/// Subcommands for Azure policy operations.
#[derive(Subcommand, Debug, Clone)]
pub enum AzurePolicyCommand {
    /// Manage Azure policy definitions.
    Definition(AzurePolicyDefinitionArgs),
}

impl AzurePolicyArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

impl AzurePolicyCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzurePolicyCommand::Definition(args) => args.invoke().await,
        }
    }
}
