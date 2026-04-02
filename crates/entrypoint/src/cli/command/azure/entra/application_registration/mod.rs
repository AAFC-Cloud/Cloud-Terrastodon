pub mod azure_entra_application_registration;
pub mod azure_entra_application_registration_browse;
pub mod azure_entra_application_registration_list;
pub mod azure_entra_application_registration_show;

pub use azure_entra_application_registration::AzureEntraApplicationRegistrationCommand;
pub use azure_entra_application_registration_browse::AzureEntraApplicationRegistrationBrowseArgs;
pub use azure_entra_application_registration_list::AzureEntraApplicationRegistrationListArgs;
pub use azure_entra_application_registration_show::AzureEntraApplicationRegistrationShowArgs;
use clap::Args;
use eyre::Result;

/// Entra application registration subcommands.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationArgs {
    #[command(subcommand)]
    pub command: AzureEntraApplicationRegistrationCommand,
}

impl AzureEntraApplicationRegistrationArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}