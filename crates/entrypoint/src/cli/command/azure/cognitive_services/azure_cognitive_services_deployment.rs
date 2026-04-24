use super::AzureCognitiveServicesDeploymentListArgs;
use super::AzureCognitiveServicesDeploymentShowArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Query Azure Cognitive Services deployments.
#[derive(Args, Debug, Clone)]
pub struct AzureCognitiveServicesDeploymentArgs {
    #[command(subcommand)]
    pub command: AzureCognitiveServicesDeploymentCommand,
}

impl AzureCognitiveServicesDeploymentArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureCognitiveServicesDeploymentCommand {
    /// List deployments for a Cognitive Services account, or all accounts when omitted.
    List(AzureCognitiveServicesDeploymentListArgs),
    /// Show a single deployment for a Cognitive Services account.
    Show(AzureCognitiveServicesDeploymentShowArgs),
}

impl AzureCognitiveServicesDeploymentCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureCognitiveServicesDeploymentCommand::List(args) => args.invoke().await,
            AzureCognitiveServicesDeploymentCommand::Show(args) => args.invoke().await,
        }
    }
}
