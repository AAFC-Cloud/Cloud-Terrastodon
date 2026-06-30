pub mod assignment;
pub mod azure_entra_role;
pub mod definition;

pub use assignment::AzureEntraRoleAssignmentArgs;
pub use azure_entra_role::AzureEntraRoleCommand;
pub use definition::AzureEntraRoleDefinitionArgs;
use eyre::Result;

/// Manage Entra directory roles.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraRoleArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraRoleCommand,
}

impl AzureEntraRoleArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
