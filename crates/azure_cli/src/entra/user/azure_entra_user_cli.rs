use super::AzureEntraUserApplicationAssignmentsArgs;
use super::AzureEntraUserAzureRoleAssignmentsArgs;
use super::AzureEntraUserBrowseArgs;
use super::AzureEntraUserListArgs;
use super::AzureEntraUserMeArgs;
use super::AzureEntraUserOwnedApplicationsArgs;
use super::AzureEntraUserOwnedGroupsArgs;
use super::AzureEntraUserPhotoArgs;
use super::AzureEntraUserSearchArgs;
use super::AzureEntraUserShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// User-related Entra (Azure AD) commands.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraUserCommand {
    /// List Entra users.
    List(AzureEntraUserListArgs),
    /// Show a single Entra user.
    Show(AzureEntraUserShowArgs),
    /// Search Entra users.
    Search(AzureEntraUserSearchArgs),
    /// Browse Entra users interactively.
    Browse(AzureEntraUserBrowseArgs),
    /// Show the signed-in Entra user.
    Me(AzureEntraUserMeArgs),
    /// Download an Entra user's profile photo.
    Photo(AzureEntraUserPhotoArgs),
    /// List groups owned by an Entra user.
    OwnedGroups(AzureEntraUserOwnedGroupsArgs),
    /// List application registrations owned by an Entra user.
    OwnedApplications(AzureEntraUserOwnedApplicationsArgs),
    /// List app role assignments granted to an Entra user.
    ApplicationAssignments(AzureEntraUserApplicationAssignmentsArgs),
    /// List direct and group-based Azure RBAC assignments for an Entra user.
    AzureRoleAssignments(AzureEntraUserAzureRoleAssignmentsArgs),
}

impl AzureEntraUserCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureEntraUserCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraUserCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraUserCommand::Search(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraUserCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraUserCommand::Me(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraUserCommand::Photo(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraUserCommand::OwnedGroups(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraUserCommand::OwnedApplications(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraUserCommand::ApplicationAssignments(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureEntraUserCommand::AzureRoleAssignments(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
