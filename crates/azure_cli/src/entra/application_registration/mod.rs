use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_entra_application_registration_browse_cli;
pub mod azure_entra_application_registration_cli;
pub mod azure_entra_application_registration_list_cli;
pub mod azure_entra_application_registration_role_cli;
pub mod azure_entra_application_registration_role_list_cli;
pub mod azure_entra_application_registration_search_cli;
pub mod azure_entra_application_registration_show_cli;

pub use azure_entra_application_registration_browse_cli::AzureEntraApplicationRegistrationBrowseArgs;
pub use azure_entra_application_registration_cli::AzureEntraApplicationRegistrationCommand;
pub use azure_entra_application_registration_list_cli::AzureEntraApplicationRegistrationListArgs;
pub use azure_entra_application_registration_role_cli::AzureEntraApplicationRegistrationRoleArgs;
pub use azure_entra_application_registration_role_list_cli::AzureEntraApplicationRegistrationRoleListArgs;
pub use azure_entra_application_registration_search_cli::AzureEntraApplicationRegistrationSearchArgs;
pub use azure_entra_application_registration_show_cli::AzureEntraApplicationRegistrationShowArgs;
use cloud_terrastodon_auth::AuthContext;
use eyre::Result;

/// Entra application registration subcommands.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraApplicationRegistrationCommand,
}

impl AzureEntraApplicationRegistrationArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.command.invoke(auth_context, executor).await
    }
}
