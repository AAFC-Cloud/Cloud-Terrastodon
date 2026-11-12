use super::AzureEntraOAuth2PermissionGrantBrowseArgs;
use super::AzureEntraOAuth2PermissionGrantClaimArgs;
use super::AzureEntraOAuth2PermissionGrantCreateArgs;
use super::AzureEntraOAuth2PermissionGrantListArgs;
use super::AzureEntraOAuth2PermissionGrantUpdateArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Entra OAuth2 delegated permission grants.
#[derive(Facet, Debug, Clone)]
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureEntraOAuth2PermissionGrantCommand::List(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureEntraOAuth2PermissionGrantCommand::Create(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureEntraOAuth2PermissionGrantCommand::Update(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureEntraOAuth2PermissionGrantCommand::Browse(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureEntraOAuth2PermissionGrantCommand::Claim(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
