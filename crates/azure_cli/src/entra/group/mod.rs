use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
mod azure_entra_group_audit_log_cli;
mod azure_entra_group_azure_role_assignments_cli;
pub mod azure_entra_group_cli;
pub mod azure_entra_group_list_cli;
mod azure_entra_group_owners_cli;
mod azure_entra_group_role_assignments_cli;
mod azure_entra_group_search_cli;
pub mod azure_entra_group_show_cli;
mod azure_entra_group_show_lookup;
mod azure_entra_group_show_lookup_result;
pub mod member;

pub use azure_entra_group_audit_log_cli::AzureEntraGroupAuditLogArgs;
pub use azure_entra_group_azure_role_assignments_cli::AzureEntraGroupAzureRoleAssignmentsArgs;
pub use azure_entra_group_cli::AzureEntraGroupCommand;
pub use azure_entra_group_list_cli::AzureEntraGroupListArgs;
pub use azure_entra_group_owners_cli::AzureEntraGroupOwnersArgs;
pub use azure_entra_group_role_assignments_cli::AzureEntraGroupRoleAssignmentsArgs;
pub use azure_entra_group_search_cli::AzureEntraGroupSearchArgs;
pub use azure_entra_group_show_cli::AzureEntraGroupShowArgs;
use cloud_terrastodon_app::CliOutput;
use eyre::Result;
pub use member::AzureEntraGroupMemberArgs;

/// Entra group subcommands
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraGroupArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraGroupCommand,
}

impl AzureEntraGroupArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
