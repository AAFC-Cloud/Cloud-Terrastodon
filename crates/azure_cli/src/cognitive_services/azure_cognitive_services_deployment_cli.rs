use super::AzureCognitiveServicesDeploymentListArgs;
use super::AzureCognitiveServicesDeploymentShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Query Azure Cognitive Services deployments.
#[derive(Facet, Debug, Clone)]
pub struct AzureCognitiveServicesDeploymentArgs {
    #[facet(figue::subcommand)]
    pub command: AzureCognitiveServicesDeploymentCommand,
}

impl AzureCognitiveServicesDeploymentArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureCognitiveServicesDeploymentCommand {
    /// List deployments for a Cognitive Services account, or all accounts when omitted.
    List(AzureCognitiveServicesDeploymentListArgs),
    /// Show a single deployment for a Cognitive Services account.
    Show(AzureCognitiveServicesDeploymentShowArgs),
}

impl AzureCognitiveServicesDeploymentCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureCognitiveServicesDeploymentCommand::List(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureCognitiveServicesDeploymentCommand::Show(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
