use super::AzureEntraPrincipalListArgs;
use super::AzureEntraPrincipalShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Principal-related Entra (Azure AD) commands.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraPrincipalCommand {
    /// List Entra principals.
    List(AzureEntraPrincipalListArgs),
    /// Show an Entra principal by object id, display name, or user principal name.
    Show(AzureEntraPrincipalShowArgs),
}

impl AzureEntraPrincipalCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureEntraPrincipalCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraPrincipalCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
