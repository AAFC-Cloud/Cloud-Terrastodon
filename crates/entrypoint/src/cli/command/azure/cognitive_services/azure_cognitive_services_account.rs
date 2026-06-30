use super::AzureCognitiveServicesDeploymentArgs;
use super::AzureCognitiveServicesListArgs;
use super::AzureCognitiveServicesShowArgs;
use eyre::Result;

/// Manage Azure Cognitive Services accounts.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureCognitiveServicesAccountArgs {
    #[facet(figue::subcommand)]
    pub command: AzureCognitiveServicesAccountCommand,
}

impl AzureCognitiveServicesAccountArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureCognitiveServicesAccountCommand {
    /// List Azure Cognitive Services accounts.
    List(AzureCognitiveServicesListArgs),
    /// Show a single Azure Cognitive Services account by resource id or resource name.
    Show(AzureCognitiveServicesShowArgs),
    /// Query deployments for Azure Cognitive Services accounts.
    Deployment(AzureCognitiveServicesDeploymentArgs),
}

impl AzureCognitiveServicesAccountCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureCognitiveServicesAccountCommand::List(args) => args.invoke().await,
            AzureCognitiveServicesAccountCommand::Show(args) => args.invoke().await,
            AzureCognitiveServicesAccountCommand::Deployment(args) => args.invoke().await,
        }
    }
}
