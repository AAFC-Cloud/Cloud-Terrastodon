use crate::service_endpoint::list::AzureDevOpsServiceEndpointListArgs;
use crate::service_endpoint::show::AzureDevOpsServiceEndpointShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps service endpoint-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsServiceEndpointArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsServiceEndpointCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsServiceEndpointCommand {
    /// List Azure DevOps service endpoints in a project.
    List(AzureDevOpsServiceEndpointListArgs),
    /// Show details for a single service endpoint.
    Show(AzureDevOpsServiceEndpointShowArgs),
}

impl AzureDevOpsServiceEndpointArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsServiceEndpointCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsServiceEndpointCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
