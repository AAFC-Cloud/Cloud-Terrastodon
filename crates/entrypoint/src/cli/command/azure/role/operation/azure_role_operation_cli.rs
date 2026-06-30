use super::AzureRoleOperationBrowseArgs;
use super::AzureRoleOperationListArgs;
use eyre::Result;

/// Subcommands for Azure provider operation metadata.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureRoleOperationCommand {
    /// List all Azure provider operations accessible to the account.
    List(AzureRoleOperationListArgs),
    /// Browse Azure provider operations interactively.
    Browse(AzureRoleOperationBrowseArgs),
}

impl AzureRoleOperationCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureRoleOperationCommand::List(args) => args.invoke().await,
            AzureRoleOperationCommand::Browse(args) => args.invoke().await,
        }
    }
}
