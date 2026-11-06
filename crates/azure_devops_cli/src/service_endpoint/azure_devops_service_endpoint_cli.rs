use crate::service_endpoint::list::AzureDevOpsServiceEndpointListArgs;
use crate::service_endpoint::show::AzureDevOpsServiceEndpointShowArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

/// Azure DevOps service endpoint-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsServiceEndpointArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsServiceEndpointCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsServiceEndpointCommand {
    /// List Azure DevOps service endpoints in a project.
    List(AzureDevOpsServiceEndpointListArgs),
    /// Show details for a single service endpoint.
    Show(AzureDevOpsServiceEndpointShowArgs),
}

impl AzureDevOpsServiceEndpointArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsServiceEndpointCommand::List(args) => {
                args.invoke(auth_context, executor).await?
            }
            AzureDevOpsServiceEndpointCommand::Show(args) => {
                args.invoke(auth_context, executor).await?
            }
        }

        Ok(())
    }
}
