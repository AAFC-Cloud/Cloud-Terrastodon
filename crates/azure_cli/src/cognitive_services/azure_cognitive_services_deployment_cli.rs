use super::AzureCognitiveServicesDeploymentListArgs;
use super::AzureCognitiveServicesDeploymentShowArgs;
use cloud_terrastodon_auth::AuthContext;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.command.invoke(auth_context, executor).await
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureCognitiveServicesDeploymentCommand::List(args) => {
                args.invoke(auth_context, executor).await
            }
            AzureCognitiveServicesDeploymentCommand::Show(args) => {
                args.invoke(auth_context, executor).await
            }
        }
    }
}
