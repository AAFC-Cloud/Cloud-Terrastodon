use super::OAuth2PermissionGrantPreset;
use super::resolve_preset_service_principals;
use super::split_scope_csv;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePrincipalArgument;
use cloud_terrastodon_azure::EntraOAuth2PermissionGrantListRequest;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::ContextCompat;
use eyre::Result;
use facet::Facet;

/// List Entra OAuth2 permission grants.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let tenant_id = authentication.tenant_id;
        let mut client_id = self.client_id;
        let mut resource_id = self.resource_id;
        if let Some(preset) = self.preset {
            let (preset_client_id, preset_resource_id) =
                resolve_preset_service_principals(preset, authentication, executor).await?;
            client_id.get_or_insert(preset_client_id);
            resource_id.get_or_insert(preset_resource_id);
        }

        let principal_id = match self.principal.as_ref() {
            Some(principal_argument) => {
                let principals = EntraPrincipalListRequest { authentication }
                    .execute(executor)
                    .await?;
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
        let grants = EntraOAuth2PermissionGrantListRequest { authentication }
            .execute(executor)
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

        Ok(CliOutput::facet(grants))
    }
}

fn principal_id_matches(actual: Option<EntraUserId>, expected: Option<EntraUserId>) -> bool {
    expected.is_none_or(|expected| actual == Some(expected))
}
