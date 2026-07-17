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
use eyre::Result;

/// Entra application registration subcommands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraApplicationRegistrationCommand,
}

impl AzureEntraApplicationRegistrationArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
