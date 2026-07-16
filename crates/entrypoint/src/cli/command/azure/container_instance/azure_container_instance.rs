use super::AzureContainerInstanceListArgs;
use super::AzureContainerInstanceShowArgs;
use eyre::Result;

/// Subcommands for Azure Container Instances.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureContainerInstanceCommand {
    /// List Azure Container Instance container groups.
    List(AzureContainerInstanceListArgs),
    /// Show a single Azure Container Instance container group by resource id, name, or IP address.
    Show(AzureContainerInstanceShowArgs),
}

/// Arguments for Azure Container Instance operations.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureContainerInstanceArgs {
    #[facet(figue::subcommand)]
    pub command: AzureContainerInstanceCommand,
}

impl AzureContainerInstanceArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

impl AzureContainerInstanceCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            Self::List(args) => args.invoke().await,
            Self::Show(args) => args.invoke().await,
        }
    }
}
