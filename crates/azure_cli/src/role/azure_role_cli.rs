use super::assignment::AzureRoleAssignmentArgs;
use super::definition::AzureRoleDefinitionArgs;
use super::operation::AzureRoleOperationArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Subcommands for Azure RBAC operations.
#[derive(facet::Facet, Debug, Clone)]
#[expect(clippy::large_enum_variant)]
#[repr(u8)]
pub enum AzureRoleCommand {
    /// Manage Azure role definitions.
    Definition(AzureRoleDefinitionArgs),
    /// Manage Azure role assignments.
    Assignment(AzureRoleAssignmentArgs),
    /// Manage Azure provider operations.
    Operation(AzureRoleOperationArgs),
}

impl AzureRoleCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self {
            AzureRoleCommand::Definition(args) => args.invoke(auth_context, cancellation).await,
            AzureRoleCommand::Assignment(args) => args.invoke(auth_context, cancellation).await,
            AzureRoleCommand::Operation(args) => args.invoke(cancellation).await,
        }
    }
}
