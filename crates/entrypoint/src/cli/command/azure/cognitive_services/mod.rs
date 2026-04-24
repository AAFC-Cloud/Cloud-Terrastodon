pub mod azure_cognitive_services;
pub mod azure_cognitive_services_account;
pub mod azure_cognitive_services_account_argument;
pub mod azure_cognitive_services_deployment;
pub mod azure_cognitive_services_deployment_list;
pub mod azure_cognitive_services_deployment_show;
pub mod azure_cognitive_services_list;
pub mod azure_cognitive_services_show;

pub use azure_cognitive_services::AzureCognitiveServicesCommand;
pub use azure_cognitive_services_account::AzureCognitiveServicesAccountArgs;
pub use azure_cognitive_services_account_argument::CognitiveServicesAccountArgument;
pub use azure_cognitive_services_deployment::AzureCognitiveServicesDeploymentArgs;
pub use azure_cognitive_services_deployment_list::AzureCognitiveServicesDeploymentListArgs;
pub use azure_cognitive_services_deployment_show::AzureCognitiveServicesDeploymentShowArgs;
pub use azure_cognitive_services_list::AzureCognitiveServicesListArgs;
pub use azure_cognitive_services_show::AzureCognitiveServicesShowArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure Cognitive Services accounts and deployments.
#[derive(Args, Debug, Clone)]
pub struct AzureCognitiveServicesArgs {
    #[command(subcommand)]
    pub command: AzureCognitiveServicesCommand,
}

impl AzureCognitiveServicesArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
