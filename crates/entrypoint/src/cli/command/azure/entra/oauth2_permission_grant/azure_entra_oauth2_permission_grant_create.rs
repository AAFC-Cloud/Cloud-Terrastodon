use super::OAuth2PermissionGrantPreset;
use super::resolve_preset_service_principals;
use super::split_scope_csv;
use cloud_terrastodon_azure::AzurePrincipalArgument;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraServicePrincipalId;
use cloud_terrastodon_azure::create_oauth2_permission_grant;
use cloud_terrastodon_azure::fetch_all_principals;
use cloud_terrastodon_azure::join_oauth2_permission_grant_scopes;
use eyre::ContextCompat;
use eyre::Result;
use std::io::Write;

/// Create an Entra OAuth2 permission grant.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantCreateArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Client service principal object id.
    #[facet(figue::named)]
    pub client_id: Option<EntraServicePrincipalId>,

    /// Principal user object id or user principal name.
    #[facet(figue::named, proxy = String)]
    pub principal: AzurePrincipalArgument<'static>,

    /// Delegated scopes to grant.
    #[facet(figue::named, figue::alias = "scopes")]
    pub scope: Vec<String>,

    /// Resource service principal object id.
    #[facet(figue::named)]
    pub resource_id: Option<EntraServicePrincipalId>,

    /// Resolve common client/resource pairs for the current tenant.
    #[facet(figue::named)]
    pub preset: Option<OAuth2PermissionGrantPreset>,
}

impl AzureEntraOAuth2PermissionGrantCreateArgs {
    pub async fn invoke(self) -> Result<()> {
        if self.preset.is_some() && (self.client_id.is_some() || self.resource_id.is_some()) {
            eyre::bail!("--preset cannot be used with --client-id or --resource-id");
        }

        let tenant_id = self.tenant.resolve().await?;
        let (client_id, resource_id) = match self.preset {
            Some(preset) => resolve_preset_service_principals(tenant_id, preset).await?,
            None => (
                self.client_id
                    .wrap_err("--client-id is required unless --preset is used")?,
                self.resource_id
                    .wrap_err("--resource-id is required unless --preset is used")?,
            ),
        };

        let scopes = split_scope_csv(&self.scope);
        if scopes.is_empty() {
            eyre::bail!("At least one --scope value is required");
        }

        let principals = fetch_all_principals(tenant_id).await?;
        let principal = self.principal.resolve(&principals).wrap_err_with(|| {
            format!(
                "Could not resolve principal '{}' in tenant {tenant_id}",
                self.principal
            )
        })?;
        let principal_id = principal.as_user().map(|user| user.id).wrap_err_with(|| {
            format!(
                "Delegated oauth2 permission grants require a user principal, got '{principal}'"
            )
        })?;

        let grant = create_oauth2_permission_grant(
            tenant_id,
            resource_id,
            client_id,
            principal_id,
            join_oauth2_permission_grant_scopes(scopes),
        )
        .await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &grant)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
