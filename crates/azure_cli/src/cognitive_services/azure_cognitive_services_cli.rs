use super::AzureCognitiveServicesAccountArgs;
use cloud_terrastodon_auth::AuthContext;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureCognitiveServicesCommand::Account(args) => {
                args.invoke(auth_context, executor).await
            }
        }
    }
}
