use super::AzureEntraOAuth2PermissionGrantBrowseArgs;
use super::AzureEntraOAuth2PermissionGrantCreateArgs;
use super::AzureEntraOAuth2PermissionGrantListArgs;
use super::AzureEntraOAuth2PermissionGrantUpdateArgs;
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
}

impl AzureEntraOAuth2PermissionGrantCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraOAuth2PermissionGrantCommand::List(args) => args.invoke().await,
            AzureEntraOAuth2PermissionGrantCommand::Create(args) => args.invoke().await,
            AzureEntraOAuth2PermissionGrantCommand::Update(args) => args.invoke().await,
            AzureEntraOAuth2PermissionGrantCommand::Browse(args) => args.invoke().await,
        }
    }
}
