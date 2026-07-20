use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraApplicationClientId;
use cloud_terrastodon_azure::EntraApplicationRegistration;
use cloud_terrastodon_azure::fetch_all_service_principals;
use cloud_terrastodon_azure::fetch_oauth2_permission_scopes;
use cloud_terrastodon_azure::search_application_registrations;
use cloud_terrastodon_config::Config;
use cloud_terrastodon_credentials::DEFAULT_PIM_GRAPH_SCOPES;
use cloud_terrastodon_credentials::PIM_APPLICATION_DISPLAY_NAME;
use cloud_terrastodon_credentials::PimConfig;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use std::collections::HashSet;
use std::io::Write;
use tracing::info;
use uuid::Uuid;

const MICROSOFT_GRAPH_APP_ID: &str = "00000003-0000-0000-c000-000000000000";
const MICROSOFT_GRAPH_SCOPE_PREFIX: &str = "https://graph.microsoft.com/";

/// Discover and configure the Cloud Terrastodon PIM app registration.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePimSetupArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzurePimSetupArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Searching for the Cloud Terrastodon PIM app registration");

        let application = find_pim_application(tenant_id).await?;
        let graph_app_id: EntraApplicationClientId = MICROSOFT_GRAPH_APP_ID.parse()?;
        let service_principals = fetch_all_service_principals(tenant_id).await?;
        let graph_service_principal = service_principals
            .iter()
            .find(|service_principal| service_principal.app_id == graph_app_id)
            .ok_or_else(|| {
                eyre::eyre!(
                    "Could not find the Microsoft Graph service principal in tenant {tenant_id}"
                )
            })?;

        let graph_scopes =
            fetch_oauth2_permission_scopes(tenant_id, graph_service_principal.id).await?;
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

        let mut config = PimConfig::load().await?;
        config.set_client_id(&tenant_id, application.app_id);
        config.save().await?;
        let output = PimSetupOutput {
            application_id: application.app_id,
            application_display_name: application.display_name,
            configured_graph_scopes: required_graph_claims
                .iter()
                .map(|claim| (*claim).to_string())
                .collect(),
            protocol_scopes: DEFAULT_PIM_GRAPH_SCOPES
                .iter()
                .filter(|scope| graph_permission_claim(scope).is_none())
                .map(|scope| (*scope).to_string())
                .collect(),
            client_id_persisted: true,
        };

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &output)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase")]
struct PimSetupOutput {
    application_id: EntraApplicationClientId,
    application_display_name: String,
    configured_graph_scopes: Vec<String>,
    protocol_scopes: Vec<String>,
    client_id_persisted: bool,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase")]
struct RequiredResourceAccess {
    resource_app_id: EntraApplicationClientId,
    resource_access: Vec<RequiredResourceAccessEntry>,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase")]
struct RequiredResourceAccessEntry {
    id: Uuid,
    #[facet(rename = "type")]
    permission_type: String,
}

async fn find_pim_application(
    tenant_id: cloud_terrastodon_azure::AzureTenantId,
) -> Result<EntraApplicationRegistration> {
    let applications =
        search_application_registrations(tenant_id, PIM_APPLICATION_DISPLAY_NAME).await?;
    let matches = applications
        .into_iter()
        .filter(|application| {
            application
                .display_name
                .eq_ignore_ascii_case(PIM_APPLICATION_DISPLAY_NAME)
        })
        .collect::<Vec<_>>();

    match matches.len() {
        0 => bail!(
            "Could not find an application registration named {PIM_APPLICATION_DISPLAY_NAME:?}"
        ),
        1 => Ok(matches.into_iter().next().expect("one match exists")),
        _ => bail!(
            "Found {} application registrations named {PIM_APPLICATION_DISPLAY_NAME:?}; use `ct az ad app search {PIM_APPLICATION_DISPLAY_NAME:?}` to inspect them",
            matches.len()
        ),
    }
}

fn parse_required_resource_access(
    application: &EntraApplicationRegistration,
) -> Result<Vec<RequiredResourceAccess>> {
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
