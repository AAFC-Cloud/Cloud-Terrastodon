use super::AzureRoleAssignmentBrowseArgs;
use super::AzureRoleAssignmentListArgs;
use super::azure_role_assignment_create_cli::AzureRoleAssignmentCreateArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Azure role assignment operations.
#[derive(Facet, Debug, Clone)]
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureRoleAssignmentCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureRoleAssignmentCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
            AzureRoleAssignmentCommand::Create(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
