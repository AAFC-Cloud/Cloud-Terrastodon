use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use reqwest::Client;
use reqwest::Url;
use std::fmt::Debug;
use std::fmt::Write;

pub const AZURE_CLIENT_ID_ENV: &str = "AZURE_CLIENT_ID";
pub const AZURE_TENANT_ID_ENV: &str = "AZURE_TENANT_ID";
pub const AZURE_FEDERATED_TOKEN_ENV: &str = "AZURE_FEDERATED_TOKEN";
pub const SERVICE_PRINCIPAL_ID_ENV: &str = "servicePrincipalId";
pub const TASK_TENANT_ID_ENV: &str = "tenantId";
pub const TASK_ID_TOKEN_ENV: &str = "idToken";

const AZURE_TOKEN_ENDPOINT_TEMPLATE: &str =
    "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token";

/// Normalized Azure DevOps/Entra workload identity inputs.
#[derive(Clone, Facet)]
pub struct WorkloadIdentityConfig {
    pub client_id: EntraApplicationClientId,
    pub tenant_id: AzureTenantId,
    #[facet(sensitive)]
    pub federated_token: String,
}

impl WorkloadIdentityConfig {
    pub fn federated_token(&self) -> &str {
        &self.federated_token
    }

    pub fn token_endpoint(&self) -> Result<Url> {
        Url::parse(
            AZURE_TOKEN_ENDPOINT_TEMPLATE
                .replace("{tenant}", &self.tenant_id.to_string())
                .as_str(),
        )
        .wrap_err("building the Entra workload-identity token endpoint")
    }
}

impl Debug for WorkloadIdentityConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorkloadIdentityConfig")
            .field("client_id", &self.client_id)
            .field("tenant_id", &self.tenant_id)
            .field("federated_token", &"***redacted***")
            .finish()
    }
}

/// Reads either the Azure SDK names or Azure DevOps task names.
pub fn load_workload_identity_config() -> Result<Option<WorkloadIdentityConfig>> {
    normalize_workload_identity_inputs(
        read_env(AZURE_CLIENT_ID_ENV)?,
        read_env(SERVICE_PRINCIPAL_ID_ENV)?,
        read_env(AZURE_TENANT_ID_ENV)?,
        read_env(TASK_TENANT_ID_ENV)?,
        read_env(AZURE_FEDERATED_TOKEN_ENV)?,
        read_env(TASK_ID_TOKEN_ENV)?,
    )
}

fn read_env(name: &str) -> Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) if !value.trim().is_empty() => Ok(Some(value)),
        Ok(_) => Ok(None),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => bail!("{name} must be valid UTF-8"),
    }
}

fn resolve_alias(
    label: &str,
    first_name: &str,
    first: Option<String>,
    second_name: &str,
    second: Option<String>,
) -> Result<Option<String>> {
    match (first, second) {
        (Some(first), Some(second)) if first != second => bail!(
            "conflicting workload identity values for {label}; {first_name} and {second_name} must match"
        ),
        (Some(value), _) | (_, Some(value)) => Ok(Some(value)),
        (None, None) => Ok(None),
    }
}

fn required_input(label: &str, value: Option<String>, names: &[&str]) -> Result<String> {
    value.ok_or_else(|| {
        eyre::eyre!(
            "workload identity is incomplete: {label} is required ({})",
            names.join(" or ")
        )
    })
}

pub(crate) fn normalize_workload_identity_inputs(
    azure_client_id: Option<String>,
    service_principal_id: Option<String>,
    azure_tenant_id: Option<String>,
    task_tenant_id: Option<String>,
    azure_federated_token: Option<String>,
    task_id_token: Option<String>,
) -> Result<Option<WorkloadIdentityConfig>> {
    let client_id = resolve_alias(
        "client ID",
        AZURE_CLIENT_ID_ENV,
        azure_client_id,
        SERVICE_PRINCIPAL_ID_ENV,
        service_principal_id,
    )?;
    let tenant_id = resolve_alias(
        "tenant ID",
        AZURE_TENANT_ID_ENV,
        azure_tenant_id,
        TASK_TENANT_ID_ENV,
        task_tenant_id,
    )?;
    let federated_token = resolve_alias(
        "federated token",
        AZURE_FEDERATED_TOKEN_ENV,
        azure_federated_token,
        TASK_ID_TOKEN_ENV,
        task_id_token,
    )?;

    if client_id.is_none() && tenant_id.is_none() && federated_token.is_none() {
        return Ok(None);
    }

    let client_id = required_input(
        "client ID",
        client_id,
        &[AZURE_CLIENT_ID_ENV, SERVICE_PRINCIPAL_ID_ENV],
    )?
    .parse::<EntraApplicationClientId>()
    .map_err(|_| eyre::eyre!("workload identity client ID is not a valid UUID"))?;
    let tenant_id = required_input(
        "tenant ID",
        tenant_id,
        &[AZURE_TENANT_ID_ENV, TASK_TENANT_ID_ENV],
    )?
    .parse::<AzureTenantId>()
    .map_err(|_| eyre::eyre!("workload identity tenant ID is not a valid UUID"))?;
    let federated_token = required_input(
        "federated token",
        federated_token,
        &[AZURE_FEDERATED_TOKEN_ENV, TASK_ID_TOKEN_ENV],
    )?;

    Ok(Some(WorkloadIdentityConfig {
        client_id,
        tenant_id,
        federated_token,
    }))
}

#[derive(Debug, Facet)]
#[facet(rename_all = "snake_case")]
struct TokenResponse {
    access_token: Option<String>,
    token_type: Option<String>,
    expires_in: Option<u64>,
    error: Option<String>,
}

pub(crate) fn resource_scope(resource: super::AzureRestResource) -> &'static str {
    match resource {
        super::AzureRestResource::MicrosoftGraph => "https://graph.microsoft.com/.default",
        super::AzureRestResource::AzureResourceManager => "https://management.azure.com/.default",
        super::AzureRestResource::AzureDevOps => "499b84ac-1321-427f-aa17-267ca6975798/.default",
    }
}

pub(crate) fn form_urlencoded(parameters: &[(&str, &str)]) -> String {
    parameters
        .iter()
        .map(|(key, value)| format!("{}={}", url_encode(key), url_encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn url_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

/// Exchanges an exposed Azure DevOps OIDC assertion for a resource token.
/// `endpoint` is injectable so tests can verify the request boundary without
/// contacting Entra; production callers use `WorkloadIdentityConfig::token_endpoint`.
pub(crate) async fn exchange_workload_identity_assertion(
    client: &Client,
    endpoint: Url,
    config: &WorkloadIdentityConfig,
    resource: super::AzureRestResource,
) -> Result<(String, u64)> {
    let body = form_urlencoded(&[
        ("client_id", &config.client_id.to_string()),
        ("scope", resource_scope(resource)),
        (
            "client_assertion_type",
            "urn:ietf:params:oauth:client-assertion-type:jwt-bearer",
        ),
        ("client_assertion", config.federated_token()),
        ("grant_type", "client_credentials"),
    ]);
    let response = client
        .post(endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .wrap_err("requesting an Entra workload-identity access token")?;
    let status = response.status();
    let response_body = response.text().await?;
    parse_token_response(status, &response_body)
}

fn parse_token_response(status: reqwest::StatusCode, response_body: &str) -> Result<(String, u64)> {
    let token = facet_json::from_str::<TokenResponse>(response_body).map_err(|error| {
        eyre::eyre!("Entra workload-identity token endpoint returned {status}: {error:?}")
    })?;
    let Some(access_token) = token.access_token else {
        bail!(
            "Entra workload-identity token exchange failed ({status}): {}",
            token.error.as_deref().unwrap_or("unknown_error"),
        );
    };
    eyre::ensure!(
        token
            .token_type
            .as_deref()
            .is_none_or(|token_type| token_type.eq_ignore_ascii_case("Bearer")),
        "Entra workload-identity token exchange returned an unsupported token type"
    );
    let expires_in = token
        .expires_in
        .ok_or_else(|| eyre::eyre!("Entra workload-identity response omitted expires_in"))?;
    Ok((access_token, expires_in))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AzureRestResource;
    use cloud_terrastodon_azure_types::AzureTenantId;
    use cloud_terrastodon_azure_types::EntraApplicationClientId;
    use std::str::FromStr;

    fn ids() -> (String, String) {
        (
            "11111111-1111-1111-1111-111111111111".to_string(),
            "22222222-2222-2222-2222-222222222222".to_string(),
        )
    }

    #[test]
    fn accepts_either_complete_environment_contract() -> Result<()> {
        let (client, tenant) = ids();
        let config = normalize_workload_identity_inputs(
            Some(client.clone()),
            None,
            Some(tenant.clone()),
            None,
            Some("assertion".to_string()),
            None,
        )?
        .expect("canonical contract should be configured");
        assert_eq!(
            config.client_id,
            EntraApplicationClientId::from_str(&client)?
        );
        assert_eq!(config.tenant_id, AzureTenantId::from_str(&tenant)?);

        let config = normalize_workload_identity_inputs(
            None,
            Some(client.clone()),
            None,
            Some(tenant.clone()),
            None,
            Some("assertion".to_string()),
        )?
        .expect("task contract should be configured");
        assert_eq!(
            config.client_id,
            EntraApplicationClientId::from_str(&client)?
        );
        assert_eq!(config.tenant_id, AzureTenantId::from_str(&tenant)?);
        Ok(())
    }

    #[test]
    fn rejects_partial_and_conflicting_inputs_without_values() -> Result<()> {
        let (_, tenant) = ids();
        let error = normalize_workload_identity_inputs(
            None,
            None,
            Some(tenant),
            None,
            Some("assertion".to_string()),
            None,
        )
        .expect_err("missing client ID should fail");
        let message = format!("{error:?}");
        assert!(message.contains("client ID"));
        assert!(!message.contains("assertion"));

        let (client, _) = ids();
        let error = normalize_workload_identity_inputs(
            Some(client),
            Some("33333333-3333-3333-3333-333333333333".to_string()),
            None,
            None,
            None,
            None,
        )
        .expect_err("conflicting aliases should fail");
        assert!(format!("{error:?}").contains("conflicting"));
        Ok(())
    }

    #[test]
    fn builds_resource_specific_exchange_form_without_logging_assertion() -> Result<()> {
        let (client, tenant) = ids();
        let config = normalize_workload_identity_inputs(
            Some(client.clone()),
            None,
            Some(tenant),
            None,
            Some("secret-assertion".to_string()),
            None,
        )?
        .unwrap();
        let form = form_urlencoded(&[
            ("client_id", &config.client_id.to_string()),
            ("scope", resource_scope(AzureRestResource::AzureDevOps)),
            ("client_assertion", config.federated_token()),
        ]);
        assert!(form.contains("client_id="));
        assert!(form.contains("499b84ac-1321-427f-aa17-267ca6975798%2F.default"));
        assert!(form.contains("secret-assertion"));
        assert_eq!(format!("{config:?}").matches("secret-assertion").count(), 0);
        Ok(())
    }

    #[test]
    fn parses_success_and_redacts_error_response_values() -> Result<()> {
        let (access_token, expires_in) = parse_token_response(
            reqwest::StatusCode::OK,
            r#"{"access_token":"token-value","token_type":"Bearer","expires_in":3600}"#,
        )?;
        assert_eq!(access_token, "token-value");
        assert_eq!(expires_in, 3600);

        let error = parse_token_response(
            reqwest::StatusCode::BAD_REQUEST,
            r#"{"error":"invalid_client","error_description":"assertion-value"}"#,
        )
        .expect_err("error response should fail");
        let message = format!("{error:?}");
        assert!(message.contains("invalid_client"));
        assert!(!message.contains("assertion-value"));
        Ok(())
    }
}
