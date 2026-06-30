use super::AzureCognitiveServicesDeploymentListArgs;
use super::AzureCognitiveServicesDeploymentShowArgs;
use eyre::Result;

/// Query Azure Cognitive Services deployments.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureCognitiveServicesDeploymentArgs {
    #[facet(figue::subcommand)]
    pub command: AzureCognitiveServicesDeploymentCommand,
}

impl AzureCognitiveServicesDeploymentArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
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
