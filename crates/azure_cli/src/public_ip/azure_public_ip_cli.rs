use super::AzurePublicIpListArgs;
use super::AzurePublicIpShowArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Azure public IP addresses.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzurePublicIpCommand {
    /// List Azure public IP addresses.
    List(AzurePublicIpListArgs),
    /// Show a single Azure public IP address by resource id, name, or IP address.
    Show(AzurePublicIpShowArgs),
}

impl AzurePublicIpCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzurePublicIpCommand::List(args) => args.invoke(auth_context, executor).await,
            AzurePublicIpCommand::Show(args) => args.invoke(auth_context, executor).await,
        }
    }
}
