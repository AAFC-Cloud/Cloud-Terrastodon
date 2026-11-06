use super::AzureContainerInstanceListArgs;
use super::AzureContainerInstanceShowArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Azure Container Instances.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureContainerInstanceCommand {
    /// List Azure Container Instance container groups.
    List(AzureContainerInstanceListArgs),
    /// Show a single Azure Container Instance container group by resource id, name, or IP address.
    Show(AzureContainerInstanceShowArgs),
}

/// Arguments for Azure Container Instance operations.
#[derive(Facet, Debug, Clone)]
pub struct AzureContainerInstanceArgs {
    #[facet(figue::subcommand)]
    pub command: AzureContainerInstanceCommand,
}

impl AzureContainerInstanceArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.command.invoke(auth_context, executor).await
    }
}

impl AzureContainerInstanceCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            Self::List(args) => args.invoke(auth_context, executor).await,
            Self::Show(args) => args.invoke(auth_context, executor).await,
        }
    }
}
