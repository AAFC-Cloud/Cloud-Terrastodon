use super::OAuth2PermissionGrantPreset;
use super::resolve_preset_service_principals;
use super::split_scope_csv;
use cloud_terrastodon_azure::AzurePrincipalArgument;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::fetch_all_principals;
use cloud_terrastodon_azure::fetch_oauth2_permission_grants;
use eyre::ContextCompat;
use eyre::Result;
use std::io::Write;

/// List Entra OAuth2 permission grants.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Filter by client service principal object id.
    #[facet(figue::named)]
    pub client_id: Option<EntraServicePrincipalObjectId>,

    /// Filter by principal user object id or user principal name.
    #[facet(figue::named)]
    pub principal: Option<AzurePrincipalArgument<'static>>,

    /// Filter by delegated scopes. Matches grants that contain all requested scopes.
    #[facet(figue::named, rename = "scope", figue::alias = "scopes")]
    pub scope: Vec<String>,

    /// Filter by resource service principal object id.
    #[facet(figue::named)]
    pub resource_id: Option<EntraServicePrincipalObjectId>,

    /// Resolve common client/resource pairs for the current tenant.
    #[facet(figue::named)]
    pub preset: Option<OAuth2PermissionGrantPreset>,
}

impl AzureEntraOAuth2PermissionGrantListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let mut client_id = self.client_id;
        let mut resource_id = self.resource_id;
        if let Some(preset) = self.preset {
            let (preset_client_id, preset_resource_id) =
                resolve_preset_service_principals(tenant_id, preset).await?;
            client_id.get_or_insert(preset_client_id);
            resource_id.get_or_insert(preset_resource_id);
        }

        let principal_id = match self.principal.as_ref() {
            Some(principal_argument) => {
                let principals = fetch_all_principals(tenant_id).await?;
                let principal = principal_argument.resolve(&principals).wrap_err_with(|| {
                    format!(
                        "Could not resolve principal '{}' in tenant {tenant_id}",
                        principal_argument
                    )
                })?;
                Some(
                    principal
                        .as_user()
                        .map(|user| user.id)
                        .wrap_err_with(|| {
                            format!(
                                "Delegated oauth2 permission grants require a user principal, got '{principal}'"
                            )
                        })?,
                )
            }
            None => None,
        };

        let scope_filter = split_scope_csv(&self.scope);
        let grants = fetch_oauth2_permission_grants(tenant_id)
            .await?
            .into_iter()
            .filter(|grant| client_id.is_none_or(|id| grant.client_id == id))
            .filter(|grant| principal_id_matches(grant.principal_id, principal_id))
            .filter(|grant| resource_id.is_none_or(|id| grant.resource_id == id))
            .filter(|grant| {
                let scopes = grant.get_scopes();
                scope_filter.iter().all(|scope| scopes.contains(scope))
            })
            .collect::<Vec<_>>();

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &grants)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

fn principal_id_matches(actual: Option<EntraUserId>, expected: Option<EntraUserId>) -> bool {
    expected.is_none_or(|expected| actual == Some(expected))
}
