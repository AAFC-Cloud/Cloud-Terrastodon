use crate::cli::azure_devops::service_endpoint::list::AzureDevOpsServiceEndpointListArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps service endpoint-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsServiceEndpointArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsServiceEndpointCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsServiceEndpointCommand {
    /// List Azure DevOps service endpoints in a project.
    List(AzureDevOpsServiceEndpointListArgs),
}

impl AzureDevOpsServiceEndpointArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsServiceEndpointCommand::List(args) => args.invoke().await?,
        }

        Ok(())
    }
}
