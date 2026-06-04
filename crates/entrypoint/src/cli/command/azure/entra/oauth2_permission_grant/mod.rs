pub mod azure_entra_oauth2_permission_grant;
pub mod azure_entra_oauth2_permission_grant_browse;
pub mod azure_entra_oauth2_permission_grant_create;
pub mod azure_entra_oauth2_permission_grant_list;
pub mod azure_entra_oauth2_permission_grant_update;

pub use azure_entra_oauth2_permission_grant::AzureEntraOAuth2PermissionGrantCommand;
pub use azure_entra_oauth2_permission_grant_browse::AzureEntraOAuth2PermissionGrantBrowseArgs;
pub use azure_entra_oauth2_permission_grant_create::AzureEntraOAuth2PermissionGrantCreateArgs;
pub use azure_entra_oauth2_permission_grant_list::AzureEntraOAuth2PermissionGrantListArgs;
pub use azure_entra_oauth2_permission_grant_update::AzureEntraOAuth2PermissionGrantUpdateArgs;
use clap::Args;
use clap::ValueEnum;
use cloud_terrastodon_azure::EntraServicePrincipal;
use cloud_terrastodon_azure::EntraServicePrincipalId;
use cloud_terrastodon_azure::fetch_all_service_principals;
use eyre::Result;
use uuid::Uuid;

const GRAPH_EXPLORER_APP_ID: &str = "de8bc8b5-d9f9-48b1-a8ad-b748da725064";
const MICROSOFT_GRAPH_APP_ID: &str = "00000003-0000-0000-c000-000000000000";

/// Entra OAuth2 permission grant subcommands.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantArgs {
    #[command(subcommand)]
    pub command: AzureEntraOAuth2PermissionGrantCommand,
}

impl AzureEntraOAuth2PermissionGrantArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OAuth2PermissionGrantPreset {
    GraphExplorer,
}

pub fn split_scope_csv(scopes: &[String]) -> Vec<&str> {
    scopes
        .iter()
        .flat_map(|scope| scope.split(','))
        .map(str::trim)
        .filter(|scope| !scope.is_empty())
        .collect()
}

pub async fn resolve_preset_service_principals(
    tenant_id: cloud_terrastodon_azure::AzureTenantId,
    preset: OAuth2PermissionGrantPreset,
) -> Result<(EntraServicePrincipalId, EntraServicePrincipalId)> {
    match preset {
        OAuth2PermissionGrantPreset::GraphExplorer => {
            let service_principals = fetch_all_service_principals(tenant_id).await?;
            let client =
                resolve_service_principal_by_app_id(&service_principals, GRAPH_EXPLORER_APP_ID)?;
            let resource =
                resolve_service_principal_by_app_id(&service_principals, MICROSOFT_GRAPH_APP_ID)?;
            Ok((client.id, resource.id))
        }
    }
}

fn resolve_service_principal_by_app_id<'a>(
    service_principals: &'a [EntraServicePrincipal],
    app_id: &str,
) -> Result<&'a EntraServicePrincipal> {
    let app_id = Uuid::parse_str(app_id)?;
    service_principals
        .iter()
        .find(|sp| sp.app_id == app_id)
        .ok_or_else(|| eyre::eyre!("Could not find service principal for app id {app_id}"))
}
