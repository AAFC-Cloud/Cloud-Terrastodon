use super::OAuth2PermissionGrantPreset;
use super::resolve_preset_service_principals;
use super::split_scope_csv;
use clap::Args;
use cloud_terrastodon_azure::AzurePrincipalArgument;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_principals;
use cloud_terrastodon_azure::fetch_oauth2_permission_grants;
use cloud_terrastodon_azure::merge_oauth2_permission_grant_scopes;
use cloud_terrastodon_azure::remove_oauth2_permission_grant;
use cloud_terrastodon_azure::update_oauth2_permission_grant;
use eyre::ContextCompat;
use eyre::Result;
use serde_json::json;
use std::io::Write;

/// Update an Entra OAuth2 permission grant.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantUpdateArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Grant id to update.
    #[arg(long)]
    pub id: Option<String>,

    /// Resolve common client/resource pairs for the current tenant.
    #[arg(long, conflicts_with = "id")]
    pub preset: Option<OAuth2PermissionGrantPreset>,

    /// Principal user object id or user principal name. Required when using --preset instead of --id.
    #[arg(long, conflicts_with = "id")]
    pub principal: Option<AzurePrincipalArgument<'static>>,

    /// Scopes to add to the grant.
    #[arg(long = "add-scope", alias = "add-scopes", value_delimiter = ',')]
    pub add_scope: Vec<String>,

    /// Scopes to remove from the grant.
    #[arg(long = "remove-scope", alias = "remove-scopes", value_delimiter = ',')]
    pub remove_scope: Vec<String>,
}

impl AzureEntraOAuth2PermissionGrantUpdateArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let add_scope = split_scope_csv(&self.add_scope);
        let remove_scope = split_scope_csv(&self.remove_scope);
        if add_scope.is_empty() && remove_scope.is_empty() {
            eyre::bail!("Specify at least one of --add-scope or --remove-scope");
        }

        let grants = fetch_oauth2_permission_grants(tenant_id).await?;
        let grant = match self.id {
            Some(id) => grants
                .into_iter()
                .find(|grant| grant.id.as_ref() == id)
                .wrap_err_with(|| format!("Could not find oauth2 permission grant {id}"))?,
            None => {
                let preset = self
                    .preset
                    .wrap_err("Use --id or (--preset and --principal)")?;
                let principal = self
                    .principal
                    .wrap_err("--principal is required when using --preset")?;
                let principals = fetch_all_principals(tenant_id).await?;
                let principal = principal.resolve(&principals).wrap_err_with(|| {
                    format!(
                        "Could not resolve principal '{}' in tenant {tenant_id}",
                        principal
                    )
                })?;
                let principal_id = principal
                    .as_user()
                    .map(|user| user.id)
                    .wrap_err_with(|| {
                        format!(
                            "Delegated oauth2 permission grants require a user principal, got '{principal}'"
                        )
                    })?;
                let (client_id, resource_id) =
                    resolve_preset_service_principals(tenant_id, preset).await?;
                grants
                    .into_iter()
                    .find(|grant| {
                        grant.resource_id == resource_id
                            && grant.client_id == client_id
                            && grant.principal_id == Some(principal_id)
                    })
                    .wrap_err(
                        "Could not find oauth2 permission grant for the requested preset and principal",
                    )?
            }
        };

        let merged_scope = merge_oauth2_permission_grant_scopes(
            &grant.scope,
            add_scope.iter().copied(),
            remove_scope.iter().copied(),
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        if merged_scope.is_empty() {
            remove_oauth2_permission_grant(tenant_id, grant.id.clone()).await?;
            serde_json::to_writer_pretty(
                &mut handle,
                &json!({
                    "deleted": true,
                    "id": grant.id,
                }),
            )?;
            handle.write_all(b"\n")?;
            return Ok(());
        }

        let () = update_oauth2_permission_grant(tenant_id, grant.id.clone(), merged_scope.clone())
            .await?;
        serde_json::to_writer_pretty(
            &mut handle,
            &json!({
                "deleted": false,
                "id": grant.id,
                "scope": merged_scope,
            }),
        )?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
