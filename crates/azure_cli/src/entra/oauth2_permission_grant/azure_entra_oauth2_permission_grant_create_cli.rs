use super::OAuth2PermissionGrantPreset;
use super::resolve_preset_service_principals;
use super::split_scope_csv;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePrincipalArgument;
use cloud_terrastodon_azure::EntraOAuth2PermissionGrantCreateRequest;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure::join_oauth2_permission_grant_scopes;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::ContextCompat;
use eyre::Result;
use facet::Facet;

/// Create an Entra OAuth2 permission grant.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantCreateArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Client service principal object id.
    #[facet(figue::named)]
    pub client_id: Option<EntraServicePrincipalObjectId>,

    /// Principal user object id or user principal name.
    #[facet(figue::named, proxy = String)]
    pub principal: AzurePrincipalArgument<'static>,

    /// Delegated scopes to grant.
    #[facet(figue::named, figue::alias = "scopes")]
    pub scope: Vec<String>,

    /// Resource service principal object id.
    #[facet(figue::named)]
    pub resource_id: Option<EntraServicePrincipalObjectId>,

    /// Resolve common client/resource pairs for the current tenant.
    #[facet(figue::named)]
    pub preset: Option<OAuth2PermissionGrantPreset>,
}

impl AzureEntraOAuth2PermissionGrantCreateArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        if self.preset.is_some() && (self.client_id.is_some() || self.resource_id.is_some()) {
            eyre::bail!("--preset cannot be used with --client-id or --resource-id");
        }

        let authentication = self.authentication.resolve(executor).await?;
        let tenant_id = authentication.tenant_id;
        let (client_id, resource_id) = match self.preset {
            Some(preset) => {
                resolve_preset_service_principals(preset, authentication, executor).await?
            }
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

        let principals = EntraPrincipalListRequest { authentication }
            .execute(executor)
            .await?;
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

        let grant = EntraOAuth2PermissionGrantCreateRequest {
            authentication,
            resource_id: (resource_id),
            client_id: (client_id),
            principal_id: (principal_id),
            scope: (join_oauth2_permission_grant_scopes(scopes)),
        }
        .execute(executor)
        .await?;

        Ok(CliOutput::facet(grant))
    }
}
