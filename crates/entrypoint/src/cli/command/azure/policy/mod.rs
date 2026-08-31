use cloud_terrastodon_credentials::AuthContext;
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
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePolicyArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePolicyCommand,
}

/// Subcommands for Azure policy operations.
#[derive(facet::Facet, Debug, Clone)]
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}

impl AzurePolicyCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzurePolicyCommand::Assignment(args) => args.invoke(auth_context).await,
            AzurePolicyCommand::Definition(args) => args.invoke(auth_context).await,
            AzurePolicyCommand::SetDefinition(args) => args.invoke(auth_context).await,
        }
    }
}
