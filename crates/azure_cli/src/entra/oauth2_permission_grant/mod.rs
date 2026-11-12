use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use facet::Facet;
use std::sync::Arc;
pub mod azure_entra_oauth2_permission_grant_browse_cli;
pub mod azure_entra_oauth2_permission_grant_claim_cli;
pub mod azure_entra_oauth2_permission_grant_claim_list_cli;
mod azure_entra_oauth2_permission_grant_claim_list_output;
pub mod azure_entra_oauth2_permission_grant_cli;
pub mod azure_entra_oauth2_permission_grant_create_cli;
pub mod azure_entra_oauth2_permission_grant_list_cli;
pub mod azure_entra_oauth2_permission_grant_update_cli;

pub use azure_entra_oauth2_permission_grant_browse_cli::AzureEntraOAuth2PermissionGrantBrowseArgs;
pub use azure_entra_oauth2_permission_grant_claim_cli::AzureEntraOAuth2PermissionGrantClaimArgs;
pub use azure_entra_oauth2_permission_grant_claim_list_cli::AzureEntraOAuth2PermissionGrantClaimListArgs;
pub use azure_entra_oauth2_permission_grant_cli::AzureEntraOAuth2PermissionGrantCommand;
pub use azure_entra_oauth2_permission_grant_create_cli::AzureEntraOAuth2PermissionGrantCreateArgs;
pub use azure_entra_oauth2_permission_grant_list_cli::AzureEntraOAuth2PermissionGrantListArgs;
pub use azure_entra_oauth2_permission_grant_update_cli::AzureEntraOAuth2PermissionGrantUpdateArgs;
use cloud_terrastodon_azure::EntraApplicationClientId;
use cloud_terrastodon_azure::EntraServicePrincipal;
use cloud_terrastodon_azure::EntraServicePrincipalListRequest;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use eyre::ContextCompat;
use eyre::Result;

const GRAPH_EXPLORER_APP_ID: &str = "de8bc8b5-d9f9-48b1-a8ad-b748da725064";
const MICROSOFT_GRAPH_APP_ID: &str = "00000003-0000-0000-c000-000000000000";

/// Entra OAuth2 permission grant subcommands.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraOAuth2PermissionGrantCommand,
}

impl AzureEntraOAuth2PermissionGrantArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}

#[derive(Debug, Clone, Copy, Facet)]
#[repr(u8)]
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
    preset: OAuth2PermissionGrantPreset,
    authentication: cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<(EntraServicePrincipalObjectId, EntraServicePrincipalObjectId)> {
    match preset {
        OAuth2PermissionGrantPreset::GraphExplorer => {
            let service_principals = EntraServicePrincipalListRequest { authentication }
                .execute(executor)
                .await?;
            let client =
                resolve_service_principal_by_app_id(&service_principals, GRAPH_EXPLORER_APP_ID)?;
            let resource =
                resolve_service_principal_by_app_id(&service_principals, MICROSOFT_GRAPH_APP_ID)?;
            Ok((client.id, resource.id))
        }
    }
}

fn resolve_service_principal_by_app_id<'a>(
    service_principals: &'a [Arc<EntraServicePrincipal>],
    app_id: &str,
) -> Result<&'a EntraServicePrincipal> {
    let app_id = app_id.parse::<EntraApplicationClientId>()?;
    service_principals
        .iter()
        .find(|sp| sp.app_id == app_id)
        .map(Arc::as_ref)
        .wrap_err_with(|| format!("Could not find service principal for app id {app_id}"))
}
