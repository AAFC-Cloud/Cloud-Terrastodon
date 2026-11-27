use super::azure_entra_group_audit_log_cli::AzureEntraGroupAuditLogArgs;
use super::azure_entra_group_azure_role_assignments_cli::AzureEntraGroupAzureRoleAssignmentsArgs;
use super::azure_entra_group_list_cli::AzureEntraGroupListArgs;
use super::azure_entra_group_owners_cli::AzureEntraGroupOwnersArgs;
use super::azure_entra_group_role_assignments_cli::AzureEntraGroupRoleAssignmentsArgs;
use super::azure_entra_group_search_cli::AzureEntraGroupSearchArgs;
use super::azure_entra_group_show_cli::AzureEntraGroupShowArgs;
use super::member::AzureEntraGroupMemberArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Entra group top-level subcommands.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraGroupCommand {
    /// List Entra groups.
    List(AzureEntraGroupListArgs),
    /// Show an Entra group by id.
    Show(AzureEntraGroupShowArgs),
    /// Operations on group members.
    Member(AzureEntraGroupMemberArgs),
    /// Search groups by object ID or display-name tokens.
    Search(AzureEntraGroupSearchArgs),
    /// List the owners of a group.
    Owners(AzureEntraGroupOwnersArgs),
    /// List directory audit events targeting a group.
    AuditLog(AzureEntraGroupAuditLogArgs),
    /// List the Entra directory roles assigned to a group.
    RoleAssignments(AzureEntraGroupRoleAssignmentsArgs),
    /// List the Azure RBAC roles assigned to a group.
    AzureRoleAssignments(AzureEntraGroupAzureRoleAssignmentsArgs),
}

impl AzureEntraGroupCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureEntraGroupCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraGroupCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraGroupCommand::Member(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraGroupCommand::Search(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraGroupCommand::Owners(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraGroupCommand::AuditLog(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraGroupCommand::RoleAssignments(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraGroupCommand::AzureRoleAssignments(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
