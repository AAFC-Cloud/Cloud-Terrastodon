use super::AzureResourceGroupBrowseArgs;
use super::AzureResourceGroupListArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for managing Azure resource groups.
#[derive(Facet, Debug, Clone)]
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
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureResourceGroupCommand::List(args) => args.invoke(auth_context, executor).await,
            AzureResourceGroupCommand::Browse(args) => args.invoke(auth_context, executor).await,
        }
    }
}
