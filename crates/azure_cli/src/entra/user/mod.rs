use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
mod azure_entra_user_application_assignments_cli;
mod azure_entra_user_azure_role_assignments_cli;
pub mod azure_entra_user_browse_cli;
pub mod azure_entra_user_cli;
pub mod azure_entra_user_list_cli;
mod azure_entra_user_me_cli;
mod azure_entra_user_owned_applications_cli;
mod azure_entra_user_owned_groups_cli;
mod azure_entra_user_photo_cli;
pub mod azure_entra_user_search_cli;
pub mod azure_entra_user_show_cli;

pub use azure_entra_user_application_assignments_cli::AzureEntraUserApplicationAssignmentsArgs;
pub use azure_entra_user_azure_role_assignments_cli::AzureEntraUserAzureRoleAssignmentsArgs;
pub use azure_entra_user_browse_cli::AzureEntraUserBrowseArgs;
pub use azure_entra_user_cli::AzureEntraUserCommand;
pub use azure_entra_user_list_cli::AzureEntraUserListArgs;
pub use azure_entra_user_me_cli::AzureEntraUserMeArgs;
pub use azure_entra_user_owned_applications_cli::AzureEntraUserOwnedApplicationsArgs;
pub use azure_entra_user_owned_groups_cli::AzureEntraUserOwnedGroupsArgs;
pub use azure_entra_user_photo_cli::AzureEntraUserPhotoArgs;
pub use azure_entra_user_search_cli::AzureEntraUserSearchArgs;
pub use azure_entra_user_show_cli::AzureEntraUserShowArgs;
use eyre::Result;

/// Entra user subcommands.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraUserArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraUserCommand,
}

impl AzureEntraUserArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
