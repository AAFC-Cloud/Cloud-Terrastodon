use super::AzureEntraRoleAssignmentBrowseArgs;
use super::AzureEntraRoleAssignmentListArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Entra role assignment operations.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraRoleAssignmentCommand {
    /// List all Entra role assignments accessible to the account.
    List(AzureEntraRoleAssignmentListArgs),
    /// Browse Entra role assignments interactively.
    Browse(AzureEntraRoleAssignmentBrowseArgs),
}

impl AzureEntraRoleAssignmentCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureEntraRoleAssignmentCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraRoleAssignmentCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
