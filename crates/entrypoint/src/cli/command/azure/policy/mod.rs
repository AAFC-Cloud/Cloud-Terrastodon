pub mod assignment;
pub mod definition;
pub mod set_definition;

pub use assignment::AzurePolicyAssignmentArgs;
pub use assignment::AzurePolicyAssignmentBrowseArgs;
pub use assignment::AzurePolicyAssignmentCommand;
pub use assignment::AzurePolicyAssignmentListArgs;
use clap::Args;
use clap::Subcommand;
pub use definition::AzurePolicyDefinitionArgs;
pub use definition::AzurePolicyDefinitionBrowseArgs;
pub use definition::AzurePolicyDefinitionCommand;
pub use definition::AzurePolicyDefinitionListArgs;
use eyre::Result;
pub use set_definition::AzurePolicySetDefinitionArgs;
pub use set_definition::AzurePolicySetDefinitionBrowseArgs;
pub use set_definition::AzurePolicySetDefinitionCommand;
pub use set_definition::AzurePolicySetDefinitionListArgs;

/// Manage Azure policy resources.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicyArgs {
    #[command(subcommand)]
    pub command: AzurePolicyCommand,
}

/// Subcommands for Azure policy operations.
#[derive(Subcommand, Debug, Clone)]
pub enum AzurePolicyCommand {
    /// Manage Azure policy assignments.
    Assignment(AzurePolicyAssignmentArgs),
    /// Manage Azure policy definitions.
    Definition(AzurePolicyDefinitionArgs),
    /// Manage Azure policy set definitions.
    #[command(alias = "setdef")]
    SetDefinition(AzurePolicySetDefinitionArgs),
}

impl AzurePolicyArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

impl AzurePolicyCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzurePolicyCommand::Assignment(args) => args.invoke().await,
            AzurePolicyCommand::Definition(args) => args.invoke().await,
            AzurePolicyCommand::SetDefinition(args) => args.invoke().await,
        }
    }
}
