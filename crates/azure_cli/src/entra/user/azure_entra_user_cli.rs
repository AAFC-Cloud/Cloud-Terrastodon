use super::AzureEntraUserBrowseArgs;
use super::AzureEntraUserListArgs;
use super::AzureEntraUserSearchArgs;
use super::AzureEntraUserShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// User-related Entra (Azure AD) commands.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraUserCommand {
    /// List Entra users.
    List(AzureEntraUserListArgs),
    /// Show a single Entra user.
    Show(AzureEntraUserShowArgs),
    /// Search Entra users.
    Search(AzureEntraUserSearchArgs),
    /// Browse Entra users interactively.
    Browse(AzureEntraUserBrowseArgs),
}

impl AzureEntraUserCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureEntraUserCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraUserCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraUserCommand::Search(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraUserCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
