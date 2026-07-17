use super::AzureEntraOAuth2PermissionGrantClaimListArgs;
use eyre::Result;

/// OAuth2 delegated permission claim operations.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantClaimArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraOAuth2PermissionGrantClaimCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraOAuth2PermissionGrantClaimCommand {
    /// List delegated permission claims exposed by a resource service principal.
    List(AzureEntraOAuth2PermissionGrantClaimListArgs),
}

impl AzureEntraOAuth2PermissionGrantClaimArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

impl AzureEntraOAuth2PermissionGrantClaimCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            Self::List(args) => args.invoke().await,
        }
    }
}
