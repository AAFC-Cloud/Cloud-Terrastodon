pub mod application_registration;
pub mod azure_entra;
pub mod group;
pub mod oauth2_permission_grant;
pub mod principal;
pub mod role;
pub mod service_principal;
pub mod user;

pub use application_registration::AzureEntraApplicationRegistrationArgs;
pub use azure_entra::AzureEntraCommand;
use clap::Args;
use eyre::Result;
pub use group::AzureEntraGroupArgs;
pub use oauth2_permission_grant::AzureEntraOAuth2PermissionGrantArgs;
pub use principal::AzureEntraPrincipalArgs;
pub use role::AzureEntraRoleArgs;
pub use service_principal::AzureEntraServicePrincipalArgs;
pub use user::AzureEntraUserArgs;

/// Entra (Azure AD) related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraArgs {
    #[command(subcommand)]
    pub command: AzureEntraCommand,
}

impl AzureEntraArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
