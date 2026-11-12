use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod assignment;
pub mod definition;
pub mod set_definition;

pub use assignment::AzurePolicyAssignmentArgs;
pub use assignment::AzurePolicyAssignmentBrowseArgs;
pub use assignment::AzurePolicyAssignmentCommand;
pub use assignment::AzurePolicyAssignmentListArgs;
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
#[derive(Facet, Debug, Clone)]
pub struct AzurePolicyArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePolicyCommand,
}

/// Subcommands for Azure policy operations.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzurePolicyCommand {
    /// Manage Azure policy assignments.
    Assignment(AzurePolicyAssignmentArgs),
    /// Manage Azure policy definitions.
    Definition(AzurePolicyDefinitionArgs),
    /// Manage Azure policy set definitions.
    #[facet(figue::alias = "setdef")]
    SetDefinition(AzurePolicySetDefinitionArgs),
}

impl AzurePolicyArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}

impl AzurePolicyCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzurePolicyCommand::Assignment(args) => Box::pin(args.invoke(executor)).await,
            AzurePolicyCommand::Definition(args) => Box::pin(args.invoke(executor)).await,
            AzurePolicyCommand::SetDefinition(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
