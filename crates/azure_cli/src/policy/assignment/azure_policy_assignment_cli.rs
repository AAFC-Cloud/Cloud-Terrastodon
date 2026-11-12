use super::AzurePolicyAssignmentBrowseArgs;
use super::AzurePolicyAssignmentListArgs;
use super::AzurePolicyAssignmentShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for managing Azure policy assignments.
#[derive(Facet, Debug, Clone)]
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzurePolicyAssignmentCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzurePolicyAssignmentCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
            AzurePolicyAssignmentCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
