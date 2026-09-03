use super::assignment::AzureRoleAssignmentArgs;
use super::definition::AzureRoleDefinitionArgs;
use super::operation::AzureRoleOperationArgs;
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzureRoleCommand::Definition(args) => args.invoke(auth_context).await,
            AzureRoleCommand::Assignment(args) => args.invoke(auth_context).await,
            AzureRoleCommand::Operation(args) => args.invoke().await,
        }
    }
}
