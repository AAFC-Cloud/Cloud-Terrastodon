use super::AzureCognitiveServicesAccountArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Azure Cognitive Services.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureCognitiveServicesCommand {
    /// Manage Azure Cognitive Services accounts.
    Account(AzureCognitiveServicesAccountArgs),
}

impl AzureCognitiveServicesCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureCognitiveServicesCommand::Account(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
