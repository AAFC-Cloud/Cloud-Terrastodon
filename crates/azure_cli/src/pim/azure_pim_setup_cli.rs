use super::azure_pim_required_resource_access::AzurePimRequiredResourceAccess;
use super::azure_pim_setup_client_id_output::AzurePimSetupClientIdOutput;
use super::azure_pim_setup_output::AzurePimSetupOutput;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraApplicationClientId;
use cloud_terrastodon_azure::EntraApplicationRegistration;
use cloud_terrastodon_azure::EntraApplicationRegistrationSearchRequest;
use cloud_terrastodon_azure::EntraOAuth2PermissionScopeListRequest;
use cloud_terrastodon_azure::EntraServicePrincipalListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::info;

const PIM_APPLICATION_DISPLAY_NAME: &str = "Cloud Terrastodon PIM";
const DEFAULT_PIM_GRAPH_SCOPES: &[&str] = &[
    "https://graph.microsoft.com/User.Read",
    "https://graph.microsoft.com/PrivilegedAccess.Read.AzureAD",
    "https://graph.microsoft.com/PrivilegedAccess.ReadWrite.AzureAD",
    "offline_access",
];
const MICROSOFT_GRAPH_APP_ID: &str = "00000003-0000-0000-c000-000000000000";
const MICROSOFT_GRAPH_SCOPE_PREFIX: &str = "https://graph.microsoft.com/";

/// Inspect a PIM app registration and show how to register a named browser account.
#[derive(Facet, Debug, Clone)]
pub struct AzurePimSetupArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Explicit app client ID for local account-registration guidance without discovery.
    #[facet(figue::named)]
    pub client_id: Option<EntraApplicationClientId>,
}

impl AzurePimSetupArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let cancellation = executor.cancellation();
        if let Some(client_id) = self.client_id {
            let tenant_id = self.authentication.tenant.resolve(cancellation).await?;

            let output = AzurePimSetupClientIdOutput {
                tenant_id,
                application_id: client_id,
                account_registration_command: format!(
                    "ct account add browser --name <account-alias> --tenant {tenant_id} --client-id {client_id}"
                ),
                permissions_verified: false,
            };
            return Ok(CliOutput::facet(output));
        }

        let authentication = self.authentication.resolve(executor).await?;
        let tenant_id = authentication.tenant_id;
        info!(%tenant_id, "Searching for the Cloud Terrastodon PIM app registration");

        let application = find_pim_application(authentication, executor).await?;
        let graph_app_id: EntraApplicationClientId = MICROSOFT_GRAPH_APP_ID.parse()?;
        let service_principals = EntraServicePrincipalListRequest { authentication }
            .execute(executor)
            .await?;
        let graph_service_principal = service_principals
            .iter()
            .find(|service_principal| service_principal.app_id == graph_app_id)
            .ok_or_else(|| {
                eyre::eyre!(
                    "Could not find the Microsoft Graph service principal in tenant {tenant_id}"
                )
            })?;

        let graph_scopes = EntraOAuth2PermissionScopeListRequest {
            service_principal_id: (graph_service_principal.id),
            authentication,
        }
        .execute(executor)
        .await?;
        let required_resource_access = parse_required_resource_access(&application)?;
        let configured_scope_ids = required_resource_access
            .iter()
            .filter(|resource| resource.resource_app_id == graph_app_id)
            .flat_map(|resource| resource.resource_access.iter())
            .filter(|permission| permission.permission_type.eq_ignore_ascii_case("Scope"))
            .map(|permission| permission.id)
            .collect::<HashSet<_>>();

        let required_graph_claims = DEFAULT_PIM_GRAPH_SCOPES
            .iter()
            .filter_map(|scope| graph_permission_claim(scope))
            .collect::<Vec<_>>();
        let missing_claim_definitions = required_graph_claims
            .iter()
            .filter(|claim| !graph_scopes.iter().any(|scope| scope.value == **claim))
            .map(|claim| (*claim).to_string())
            .collect::<Vec<_>>();
        let missing_configured_claims = required_graph_claims
            .iter()
            .filter_map(|claim| {
                let scope = graph_scopes.iter().find(|scope| scope.value == **claim)?;
                (!configured_scope_ids.contains(&scope.id)).then_some((*claim).to_string())
            })
            .collect::<Vec<_>>();

        if !missing_claim_definitions.is_empty() || !missing_configured_claims.is_empty() {
            let mut problems = Vec::new();
            if !missing_claim_definitions.is_empty() {
                problems.push(format!(
                    "not exposed by Microsoft Graph: {}",
                    missing_claim_definitions.join(", ")
                ));
            }
            if !missing_configured_claims.is_empty() {
                problems.push(format!(
                    "not declared by the app registration: {}",
                    missing_configured_claims.join(", ")
                ));
            }
            bail!(
                "Cloud Terrastodon PIM app registration is missing required Graph permissions: {}",
                problems.join("; ")
            );
        }
        let output = AzurePimSetupOutput {
            tenant_id,
            application_id: application.app_id,
            application_display_name: application.display_name.clone(),
            configured_graph_scopes: required_graph_claims
                .iter()
                .map(|claim| (*claim).to_string())
                .collect(),
            protocol_scopes: DEFAULT_PIM_GRAPH_SCOPES
                .iter()
                .filter(|scope| graph_permission_claim(scope).is_none())
                .map(|scope| (*scope).to_string())
                .collect(),
            account_registration_command: format!(
                "ct account add browser --name <account-alias> --tenant {tenant_id} --client-id {}",
                application.app_id
            ),
        };

        Ok(CliOutput::facet(output))
    }
}

async fn find_pim_application(
    authentication: AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<Arc<EntraApplicationRegistration>> {
    let applications = EntraApplicationRegistrationSearchRequest {
        search_term: (PIM_APPLICATION_DISPLAY_NAME).into(),
        authentication,
    }
    .execute(executor)
    .await?;
    let matches = applications
        .into_iter()
        .filter(|application| {
            application
                .display_name
                .eq_ignore_ascii_case(PIM_APPLICATION_DISPLAY_NAME)
        })
        .collect::<Vec<_>>();

    match matches.len() {
        0 => {
            bail!(
                "Could not find an application registration named {PIM_APPLICATION_DISPLAY_NAME:?}"
            );
        }
        1 => matches
            .into_iter()
            .next()
            .ok_or_else(|| eyre::eyre!("Matching application registration is missing")),
        _ => {
            bail!(
                "Found {} application registrations named {PIM_APPLICATION_DISPLAY_NAME:?}; use `ct az ad app search {PIM_APPLICATION_DISPLAY_NAME:?}` to inspect them",
                matches.len()
            );
        }
    }
}

fn parse_required_resource_access(
    application: &EntraApplicationRegistration,
) -> Result<Vec<AzurePimRequiredResourceAccess>> {
    application
        .required_resource_access
        .iter()
        .map(|raw| {
            facet_json::from_str(raw.as_str()).wrap_err("parsing application API permissions")
        })
        .collect()
}

fn graph_permission_claim(scope: &str) -> Option<&str> {
    if scope == "offline_access" {
        return None;
    }
    Some(
        scope
            .strip_prefix(MICROSOFT_GRAPH_SCOPE_PREFIX)
            .unwrap_or(scope),
    )
}

#[cfg(test)]
mod tests {
    use super::graph_permission_claim;

    #[test]
    fn separates_protocol_scope_from_graph_permissions() {
        assert_eq!(graph_permission_claim("offline_access"), None);
        assert_eq!(
            graph_permission_claim("https://graph.microsoft.com/User.Read"),
            Some("User.Read")
        );
    }
}
