use super::AzureCognitiveServicesDeploymentArgs;
use super::AzureCognitiveServicesListArgs;
use super::AzureCognitiveServicesShowArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Manage Azure Cognitive Services accounts.
#[derive(Args, Debug, Clone)]
pub struct AzureCognitiveServicesAccountArgs {
    #[command(subcommand)]
    pub command: AzureCognitiveServicesAccountCommand,
}

impl AzureCognitiveServicesAccountArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

#[derive(Subcommand, Debug, Clone)]
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
