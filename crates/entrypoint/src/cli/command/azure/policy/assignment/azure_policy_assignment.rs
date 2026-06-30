use super::AzurePolicyAssignmentBrowseArgs;
use super::AzurePolicyAssignmentListArgs;
use super::AzurePolicyAssignmentShowArgs;
use eyre::Result;

/// Subcommands for managing Azure policy assignments.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzurePolicyAssignmentCommand {
    /// List all Azure policy assignments accessible to the account.
    List(AzurePolicyAssignmentListArgs),
    /// Browse Azure policy assignments in an interactive manner.
    Browse(AzurePolicyAssignmentBrowseArgs),
    /// Show a single Azure policy assignment by id, name, or display name.
    Show(AzurePolicyAssignmentShowArgs),
}

impl AzurePolicyAssignmentCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzurePolicyAssignmentCommand::List(args) => args.invoke().await,
            AzurePolicyAssignmentCommand::Browse(args) => args.invoke().await,
            AzurePolicyAssignmentCommand::Show(args) => args.invoke().await,
        }
    }
}
