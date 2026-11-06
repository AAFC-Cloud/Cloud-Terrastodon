use super::AzureResourceBrowseArgs;
use super::AzureResourceListArgs;
use super::AzureResourceShowArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for managing Azure resources.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureResourceCommand {
    /// List all Azure resources accessible to the account.
    List(AzureResourceListArgs),
    /// Browse Azure resources in an interactive manner.
    Browse(AzureResourceBrowseArgs),
    /// Show a single Azure resource by id or by name.
    Show(AzureResourceShowArgs),
}

impl AzureResourceCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureResourceCommand::List(args) => args.invoke(auth_context, executor).await,
            AzureResourceCommand::Browse(args) => args.invoke(auth_context, executor).await,
            AzureResourceCommand::Show(args) => args.invoke(auth_context, executor).await,
        }
    }
}
