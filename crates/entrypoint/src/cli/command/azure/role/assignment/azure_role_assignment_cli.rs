use super::AzureRoleAssignmentBrowseArgs;
use super::AzureRoleAssignmentListArgs;
use super::azure_role_assignment_create_cli::AzureRoleAssignmentCreateArgs;
use eyre::Result;

/// Subcommands for Azure role assignment operations.
#[derive(facet::Facet, Debug, Clone)]
#[expect(clippy::large_enum_variant)]
#[repr(u8)]
pub enum AzureRoleAssignmentCommand {
    /// List all Azure role assignments accessible to the account.
    List(AzureRoleAssignmentListArgs),
    /// Browse Azure role assignments interactively.
    Browse(AzureRoleAssignmentBrowseArgs),
    /// Create Azure role assignments.
    Create(AzureRoleAssignmentCreateArgs),
}

impl AzureRoleAssignmentCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureRoleAssignmentCommand::List(args) => args.invoke().await,
            AzureRoleAssignmentCommand::Browse(args) => args.invoke().await,
            AzureRoleAssignmentCommand::Create(args) => args.invoke().await,
        }
    }
}
