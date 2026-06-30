pub mod assignment;
pub mod azure_role;
pub mod definition;
pub mod operation;

pub use assignment::AzureRoleAssignmentArgs;
pub use azure_role::AzureRoleCommand;
pub use definition::AzureRoleDefinitionArgs;
use eyre::Result;
pub use operation::AzureRoleOperationArgs;

/// Manage Azure role-based access control.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureRoleArgs {
    #[facet(figue::subcommand)]
    pub command: AzureRoleCommand,
}

impl AzureRoleArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
