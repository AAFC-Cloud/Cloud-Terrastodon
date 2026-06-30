use super::AzureCognitiveServicesAccountArgs;
use eyre::Result;

/// Subcommands for Azure Cognitive Services.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
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
