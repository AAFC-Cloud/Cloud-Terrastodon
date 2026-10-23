use super::AzureRoleOperationBrowseArgs;
use super::AzureRoleOperationListArgs;
use cloud_terrastodon_command::CancellationScope;
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
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<()> {
        match self {
            AzureRoleOperationCommand::List(args) => args.invoke(cancellation).await,
            AzureRoleOperationCommand::Browse(args) => args.invoke(cancellation).await,
        }
    }
}
