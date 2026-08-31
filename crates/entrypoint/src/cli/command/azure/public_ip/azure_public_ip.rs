use cloud_terrastodon_credentials::AuthContext;
use super::AzurePublicIpListArgs;
use super::AzurePublicIpShowArgs;
use eyre::Result;

/// Subcommands for Azure public IP addresses.
#[derive(facet::Facet, Debug, Clone)]
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
    ) -> Result<()> {
        match self {
            AzurePublicIpCommand::List(args) => args.invoke(auth_context).await,
            AzurePublicIpCommand::Show(args) => args.invoke(auth_context).await,
        }
    }
}
