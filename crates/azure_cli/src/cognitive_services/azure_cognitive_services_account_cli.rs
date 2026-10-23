use super::AzureCognitiveServicesDeploymentArgs;
use super::AzureCognitiveServicesListArgs;
use super::AzureCognitiveServicesShowArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Manage Azure Cognitive Services accounts.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureCognitiveServicesAccountArgs {
    #[facet(figue::subcommand)]
    pub command: AzureCognitiveServicesAccountCommand,
}

impl AzureCognitiveServicesAccountArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        self.command.invoke(auth_context, cancellation).await
    }
}

#[derive(facet::Facet, Debug, Clone)]
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self {
            AzureCognitiveServicesAccountCommand::List(args) => {
                args.invoke(auth_context, cancellation).await
            }
            AzureCognitiveServicesAccountCommand::Show(args) => {
                args.invoke(auth_context, cancellation).await
            }
            AzureCognitiveServicesAccountCommand::Deployment(args) => {
                args.invoke(auth_context, cancellation).await
            }
        }
    }
}
