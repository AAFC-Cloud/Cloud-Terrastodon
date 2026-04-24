use super::AzureCognitiveServicesAccountArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for Azure Cognitive Services.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureCognitiveServicesCommand {
    /// Manage Azure Cognitive Services accounts.
    Account(AzureCognitiveServicesAccountArgs),
}

impl AzureCognitiveServicesCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureCognitiveServicesCommand::Account(args) => args.invoke().await,
        }
    }
}
