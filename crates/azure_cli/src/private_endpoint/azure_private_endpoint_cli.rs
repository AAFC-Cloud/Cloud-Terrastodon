use super::AzurePrivateEndpointListArgs;
use super::AzurePrivateEndpointShowArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

/// Subcommands for Azure private endpoints.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzurePrivateEndpointCommand {
    /// List Azure private endpoints.
    List(AzurePrivateEndpointListArgs),
    /// Show a single Azure private endpoint by resource id, name, NIC id, custom NIC name, target resource id, private IP, or FQDN.
    Show(AzurePrivateEndpointShowArgs),
}

impl AzurePrivateEndpointCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzurePrivateEndpointCommand::List(args) => args.invoke(auth_context, executor).await,
            AzurePrivateEndpointCommand::Show(args) => args.invoke(auth_context, executor).await,
        }
    }
}
