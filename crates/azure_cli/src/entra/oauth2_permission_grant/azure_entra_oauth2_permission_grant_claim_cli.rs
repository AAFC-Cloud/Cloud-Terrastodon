use super::AzureEntraOAuth2PermissionGrantClaimListArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// OAuth2 delegated permission claim operations.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantClaimArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraOAuth2PermissionGrantClaimCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraOAuth2PermissionGrantClaimCommand {
    /// List delegated permission claims exposed by a resource service principal.
    List(AzureEntraOAuth2PermissionGrantClaimListArgs),
}

impl AzureEntraOAuth2PermissionGrantClaimArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.command.invoke(auth_context, executor).await
    }
}

impl AzureEntraOAuth2PermissionGrantClaimCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            Self::List(args) => args.invoke(auth_context, executor).await,
        }
    }
}
