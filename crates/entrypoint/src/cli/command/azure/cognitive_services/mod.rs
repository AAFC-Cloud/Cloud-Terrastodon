use cloud_terrastodon_credentials::AuthContext;
pub mod azure_cognitive_services_account_argument_cli;
pub mod azure_cognitive_services_account_cli;
pub mod azure_cognitive_services_cli;
pub mod azure_cognitive_services_deployment_cli;
pub mod azure_cognitive_services_deployment_list_cli;
pub mod azure_cognitive_services_deployment_show_cli;
pub mod azure_cognitive_services_list_cli;
pub mod azure_cognitive_services_show_cli;

pub use azure_cognitive_services_account_argument_cli::CognitiveServicesAccountArgument;
pub use azure_cognitive_services_account_cli::AzureCognitiveServicesAccountArgs;
pub use azure_cognitive_services_cli::AzureCognitiveServicesCommand;
pub use azure_cognitive_services_deployment_cli::AzureCognitiveServicesDeploymentArgs;
pub use azure_cognitive_services_deployment_list_cli::AzureCognitiveServicesDeploymentListArgs;
pub use azure_cognitive_services_deployment_show_cli::AzureCognitiveServicesDeploymentShowArgs;
pub use azure_cognitive_services_list_cli::AzureCognitiveServicesListArgs;
pub use azure_cognitive_services_show_cli::AzureCognitiveServicesShowArgs;
use eyre::Result;

/// Manage Azure Cognitive Services accounts and deployments.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureCognitiveServicesArgs {
    #[facet(figue::subcommand)]
    pub command: AzureCognitiveServicesCommand,
}

impl AzureCognitiveServicesArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
