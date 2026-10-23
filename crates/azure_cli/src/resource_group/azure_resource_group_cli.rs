use super::AzureResourceGroupBrowseArgs;
use super::AzureResourceGroupListArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Subcommands for managing Azure resource groups.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureResourceGroupCommand {
    /// List all Azure resource groups accessible to the account.
    List(AzureResourceGroupListArgs),
    /// Browse Azure resource groups in an interactive manner.
    Browse(AzureResourceGroupBrowseArgs),
}

impl AzureResourceGroupCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self {
            AzureResourceGroupCommand::List(args) => args.invoke(auth_context, cancellation).await,
            AzureResourceGroupCommand::Browse(args) => {
                args.invoke(auth_context, cancellation).await
            }
        }
    }
}
