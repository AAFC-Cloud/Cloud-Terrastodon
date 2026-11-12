use super::AzureCognitiveServicesDeploymentArgs;
use super::AzureCognitiveServicesListArgs;
use super::AzureCognitiveServicesShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Manage Azure Cognitive Services accounts.
#[derive(Facet, Debug, Clone)]
pub struct AzureCognitiveServicesAccountArgs {
    #[facet(figue::subcommand)]
    pub command: AzureCognitiveServicesAccountCommand,
}

impl AzureCognitiveServicesAccountArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}

#[derive(Facet, Debug, Clone)]
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureCognitiveServicesAccountCommand::List(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureCognitiveServicesAccountCommand::Show(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureCognitiveServicesAccountCommand::Deployment(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
