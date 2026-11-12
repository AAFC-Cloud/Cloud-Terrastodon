use super::AzureContainerInstanceListArgs;
use super::AzureContainerInstanceShowArgs;
use cloud_terrastodon_app::CliOutput;
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}

impl AzureContainerInstanceCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            Self::List(args) => Box::pin(args.invoke(executor)).await,
            Self::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
