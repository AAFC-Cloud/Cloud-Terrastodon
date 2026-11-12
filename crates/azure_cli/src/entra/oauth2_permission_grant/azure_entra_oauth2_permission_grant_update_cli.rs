use super::OAuth2PermissionGrantPreset;
use super::resolve_preset_service_principals;
use super::split_scope_csv;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePrincipalArgument;
use cloud_terrastodon_azure::EntraOAuth2PermissionGrantListRequest;
use cloud_terrastodon_azure::EntraOAuth2PermissionGrantRemoveRequest;
use cloud_terrastodon_azure::EntraOAuth2PermissionGrantUpdateRequest;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_azure::merge_oauth2_permission_grant_scopes;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::ContextCompat;
use eyre::Result;
use facet::Facet;

/// Update an Entra OAuth2 permission grant.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantUpdateArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Grant id to update.
    #[facet(figue::named)]
    pub id: Option<String>,

    /// Resolve common client/resource pairs for the current tenant.
    #[facet(figue::named)]
    pub preset: Option<OAuth2PermissionGrantPreset>,

    /// Principal user object id or user principal name. Required when using --preset instead of --id.
    #[facet(figue::named)]
    pub principal: Option<AzurePrincipalArgument<'static>>,

    /// Scopes to add to the grant.
    #[facet(figue::named, figue::alias = "add-scopes")]
    pub add_scope: Vec<String>,

    /// Scopes to remove from the grant.
    #[facet(figue::named, figue::alias = "remove-scopes")]
    pub remove_scope: Vec<String>,
}

impl AzureEntraOAuth2PermissionGrantUpdateArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        if self.id.is_some() && (self.preset.is_some() || self.principal.is_some()) {
            eyre::bail!("--id cannot be used with --preset or --principal");
        }

        let authentication = self.authentication.resolve(executor).await?;
        let tenant_id = authentication.tenant_id;
        let add_scope = split_scope_csv(&self.add_scope);
        let remove_scope = split_scope_csv(&self.remove_scope);
        if add_scope.is_empty() && remove_scope.is_empty() {
            eyre::bail!("Specify at least one of --add-scope or --remove-scope");
        }

        let grants = EntraOAuth2PermissionGrantListRequest { authentication }
            .execute(executor)
            .await?;
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
                let principals = EntraPrincipalListRequest { authentication }
                    .execute(executor)
                    .await?;
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
                    resolve_preset_service_principals(preset, authentication, executor).await?;
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

        if merged_scope.is_empty() {
            #[derive(Facet)]
            struct DeletedResponse {
                deleted: bool,
                id: cloud_terrastodon_azure::OAuth2PermissionGrantId,
            }

            EntraOAuth2PermissionGrantRemoveRequest {
                authentication,
                id: (grant.id.clone()),
            }
            .execute(executor)
            .await?;
            return Ok(CliOutput::facet(DeletedResponse {
                deleted: true,
                id: grant.id.clone(),
            }));
        }

        #[derive(Facet)]
        struct UpdatedResponse {
            deleted: bool,
            id: cloud_terrastodon_azure::OAuth2PermissionGrantId,
            scope: String,
        }

        let () = EntraOAuth2PermissionGrantUpdateRequest {
            authentication,
            id: (grant.id.clone()),
            scope: (merged_scope.clone()),
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(UpdatedResponse {
            deleted: false,
            id: grant.id.clone(),
            scope: merged_scope,
        }))
    }
}
