use cloud_terrastodon_credentials::AuthContext;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
    ) -> Result<()> {
        match self {
            AzureCognitiveServicesCommand::Account(args) => args.invoke(auth_context).await,
        }
    }
}
