use super::AzureEntraOAuth2PermissionGrantBrowseArgs;
use super::AzureEntraOAuth2PermissionGrantClaimArgs;
use super::AzureEntraOAuth2PermissionGrantCreateArgs;
use super::AzureEntraOAuth2PermissionGrantListArgs;
use super::AzureEntraOAuth2PermissionGrantUpdateArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Subcommands for Entra OAuth2 delegated permission grants.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraOAuth2PermissionGrantCommand {
    /// List delegated permission grants.
    List(AzureEntraOAuth2PermissionGrantListArgs),
    /// Create a delegated permission grant.
    Create(AzureEntraOAuth2PermissionGrantCreateArgs),
    /// Update an existing delegated permission grant.
    Update(AzureEntraOAuth2PermissionGrantUpdateArgs),
    /// Browse delegated permission grants interactively.
    Browse(AzureEntraOAuth2PermissionGrantBrowseArgs),
    /// Inspect delegated permission claims exposed by resource service principals.
    Claim(AzureEntraOAuth2PermissionGrantClaimArgs),
}

impl AzureEntraOAuth2PermissionGrantCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzureEntraOAuth2PermissionGrantCommand::List(args) => args.invoke(auth_context).await,
            AzureEntraOAuth2PermissionGrantCommand::Create(args) => args.invoke(auth_context).await,
            AzureEntraOAuth2PermissionGrantCommand::Update(args) => args.invoke(auth_context).await,
            AzureEntraOAuth2PermissionGrantCommand::Browse(args) => args.invoke(auth_context).await,
            AzureEntraOAuth2PermissionGrantCommand::Claim(args) => args.invoke(auth_context).await,
        }
    }
}
