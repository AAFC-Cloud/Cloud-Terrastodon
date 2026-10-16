use super::AzurePrivateEndpointListArgs;
use super::AzurePrivateEndpointShowArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
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
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self {
            AzurePrivateEndpointCommand::List(args) => {
                args.invoke(auth_context, cancellation).await
            }
            AzurePrivateEndpointCommand::Show(args) => {
                args.invoke(auth_context, cancellation).await
            }
        }
    }
}
