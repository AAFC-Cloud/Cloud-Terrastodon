use super::AzureRoleOperationBrowseArgs;
use super::AzureRoleOperationListArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Azure provider operation metadata.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureRoleOperationCommand {
    /// List all Azure provider operations accessible to the account.
    List(AzureRoleOperationListArgs),
    /// Browse Azure provider operations interactively.
    Browse(AzureRoleOperationBrowseArgs),
}

impl AzureRoleOperationCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureRoleOperationCommand::List(args) => args.invoke(auth_context, executor).await,
            AzureRoleOperationCommand::Browse(args) => args.invoke(auth_context, executor).await,
        }
    }
}
