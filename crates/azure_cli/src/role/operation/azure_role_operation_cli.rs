use super::AzureRoleOperationBrowseArgs;
use super::AzureRoleOperationListArgs;
use cloud_terrastodon_app::CliOutput;
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureRoleOperationCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureRoleOperationCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
