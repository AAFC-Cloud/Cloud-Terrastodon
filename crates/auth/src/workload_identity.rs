use crate::CancellationScope;
use crate::WorkloadIdentityConfig;
use crate::workload_identity_token_response::WorkloadIdentityTokenResponse;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use eyre::Context;
use eyre::Result;
use reqwest::Client;
use reqwest::Url;
use std::fmt::Write;

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
    resource: AzureCredentialResource,
    cancellation: &CancellationScope,
) -> Result<(String, u64)> {
    cancellation
        .cancellable_scope(|_| async move {
            let body = form_urlencoded(&[
                ("client_id", &config.client_id.to_string()),
                ("scope", resource.scope()),
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
            WorkloadIdentityTokenResponse::decode(status, &response_body)
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::AzureCredentialResource;

    fn ids() -> (String, String) {
        (
            "11111111-1111-1111-1111-111111111111".to_string(),
            "22222222-2222-2222-2222-222222222222".to_string(),
        )
    }

    #[test]
    fn builds_resource_specific_exchange_form_without_logging_assertion() -> Result<()> {
        let (client, tenant) = ids();
        let config = WorkloadIdentityConfig {
            client_id: client.parse()?,
            tenant_id: tenant.parse()?,
            federated_token: "secret-assertion".to_owned(),
        };
        let form = form_urlencoded(&[
            ("client_id", &config.client_id.to_string()),
            ("scope", AzureCredentialResource::AzureDevOps.scope()),
            ("client_assertion", config.federated_token()),
        ]);
        assert!(form.contains("client_id="));
        assert!(form.contains("499b84ac-1321-427f-aa17-267ca6975798%2F.default"));
        assert!(form.contains("secret-assertion"));
        assert_eq!(format!("{config:?}").matches("secret-assertion").count(), 0);
        Ok(())
    }
}
