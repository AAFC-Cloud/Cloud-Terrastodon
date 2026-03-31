use cloud_terrastodon_azure_types::AzureApplicationGatewayResourceBackendHealthResponse;
use cloud_terrastodon_azure_types::AzureApplicationGatewayResourceId;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::SerializableRestResponse;
use eyre::OptionExt;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

const APPLICATION_GATEWAY_BACKEND_HEALTH_API_VERSION: &str = "2025-03-01";
const APPLICATION_GATEWAY_BACKEND_HEALTH_CACHE_DURATION: Duration = Duration::from_secs(30);
const APPLICATION_GATEWAY_BACKEND_HEALTH_HEADERS: &str =
    r#"{"content-type":"application/x-www-form-urlencoded; charset=UTF-8"}"#;
const MAX_POLL_ATTEMPTS: usize = 10;

/// <https://learn.microsoft.com/en-us/rest/api/application-gateway/application-gateways/backend-health?view=rest-application-gateway-2025-05-01&tabs=HTTP>
#[must_use = "This is a future request, you must .await it"]
pub struct AzureApplicationGatewayResourceBackendHealthRequest {
    pub tenant_id: AzureTenantId,
    pub application_gateway_id: AzureApplicationGatewayResourceId,
}

pub fn fetch_application_gateway_backend_health(
    tenant_id: AzureTenantId,
    application_gateway_id: AzureApplicationGatewayResourceId,
) -> AzureApplicationGatewayResourceBackendHealthRequest {
    AzureApplicationGatewayResourceBackendHealthRequest {
        tenant_id,
        application_gateway_id,
    }
}

#[async_trait]
impl CacheableCommand for AzureApplicationGatewayResourceBackendHealthRequest {
    type Output = AzureApplicationGatewayResourceBackendHealthResponse;

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter([
                "az",
                "application_gateway",
                "backend_health",
                self.tenant_id.to_string().as_str(),
                self.application_gateway_id
                    .resource_group_id
                    .subscription_id
                    .to_string()
                    .as_str(),
                self.application_gateway_id
                    .resource_group_id
                    .resource_group_name
                    .as_ref(),
                self.application_gateway_id
                    .azure_application_gateway_resource_name
                    .as_ref(),
            ]),
            valid_for: APPLICATION_GATEWAY_BACKEND_HEALTH_CACHE_DURATION,
        }
    }

    async fn run(self) -> Result<Self::Output> {
        info!(
            tenant_id = %self.tenant_id,
            application_gateway_id = %self.application_gateway_id.expanded_form(),
            "Fetching Azure application gateway backend health"
        );

        let backend_health_url = build_backend_health_url(&self.application_gateway_id);
        let initial_response = {
            let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
            cmd.args([
                "rest",
                "--method",
                "POST",
                "--url",
                &backend_health_url,
                "--body",
                "{}",
                "--headers",
                APPLICATION_GATEWAY_BACKEND_HEALTH_HEADERS,
                "--output-format",
                "json",
                "--tenant",
                self.tenant_id.to_string().as_str(),
            ]);
            cmd.run::<SerializableRestResponse>().await?
        };

        if !initial_response.ok {
            bail!(
                "Backend health request failed with status {} ({})",
                initial_response.status,
                initial_response
                    .reason_phrase
                    .as_deref()
                    .unwrap_or("Unknown error")
            );
        }

        match initial_response.status {
            200 => parse_backend_health_response(initial_response),
            202 => {
                let mut next_url = initial_response
                    .header("location")
                    .map(str::to_owned)
                    .ok_or_eyre(
                        "Backend health request returned 202 Accepted without a Location header",
                    )?;

                for attempt in 0..MAX_POLL_ATTEMPTS {
                    let response = {
                        let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
                        cmd.args([
                            "rest",
                            "--method",
                            "GET",
                            "--url",
                            &next_url,
                            "--output-format",
                            "json",
                            "--tenant",
                            self.tenant_id.to_string().as_str(),
                        ]);
                        cmd.run::<SerializableRestResponse>().await?
                    };

                    if !response.ok {
                        bail!(
                            "Backend health poll failed with status {} ({})",
                            response.status,
                            response.reason_phrase.as_deref().unwrap_or("Unknown error")
                        );
                    }

                    match response.status {
                        200 => return parse_backend_health_response(response),
                        202 => {
                            next_url = response
                                .header("location")
                                .map(str::to_owned)
                                .ok_or_eyre("Backend health poll returned 202 Accepted without a Location header")?;
                            info!(
                                attempt = attempt + 1,
                                "Backend health still pending, polling again"
                            );
                        }
                        status => {
                            bail!(
                                "Unexpected backend health poll status {} ({})",
                                status,
                                response.reason_phrase.as_deref().unwrap_or("Unknown error")
                            );
                        }
                    }
                }

                bail!(
                    "Backend health polling did not complete after {} attempts",
                    MAX_POLL_ATTEMPTS
                )
            }
            status => bail!(
                "Unexpected backend health response status {} ({})",
                status,
                initial_response
                    .reason_phrase
                    .as_deref()
                    .unwrap_or("Unknown error")
            ),
        }
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(
    AzureApplicationGatewayResourceBackendHealthRequest
);

fn build_backend_health_url(application_gateway_id: &AzureApplicationGatewayResourceId) -> String {
    format!(
        "https://management.azure.com{}/backendhealth?api-version={}",
        application_gateway_id.expanded_form(),
        APPLICATION_GATEWAY_BACKEND_HEALTH_API_VERSION
    )
}

fn parse_backend_health_response(
    response: SerializableRestResponse,
) -> Result<AzureApplicationGatewayResourceBackendHealthResponse> {
    serde_json::from_value(response.into_json_body()?)
        .wrap_err("Deserializing Azure application gateway backend health response")
}

#[cfg(test)]
mod tests {
    use super::build_backend_health_url;
    use super::parse_backend_health_response;
    use cloud_terrastodon_azure_types::AzureApplicationGatewayResourceBackendHealthProbeErrorName;
    use cloud_terrastodon_azure_types::AzureApplicationGatewayResourceBackendHealthResponse;
    use cloud_terrastodon_azure_types::AzureApplicationGatewayResourceBackendHealthServerHealth;
    use cloud_terrastodon_azure_types::AzureApplicationGatewayResourceId;
    use cloud_terrastodon_azure_types::Scope;
    use cloud_terrastodon_credentials::RestResponseBody;
    use cloud_terrastodon_credentials::SerializableRestResponse;
    use std::collections::BTreeMap;

    #[test]
    fn builds_backend_health_url() -> eyre::Result<()> {
        let id = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/applicationGateways/my-agw"
            .parse::<AzureApplicationGatewayResourceId>()?;
        assert_eq!(
            build_backend_health_url(&id),
            "https://management.azure.com/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/applicationGateways/my-agw/backendhealth?api-version=2025-03-01"
        );
        assert_eq!(
            id.expanded_form(),
            "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/applicationGateways/my-agw"
        );
        Ok(())
    }

    #[test]
    fn location_header_lookup_is_case_insensitive() {
        let response = SerializableRestResponse {
            status: 202,
            ok: true,
            reason_phrase: Some("Accepted".to_string()),
            headers: BTreeMap::from([(
                String::from("Location"),
                vec![String::from("https://example.test/poll")],
            )]),
            body: RestResponseBody::Json(serde_json::Value::Null),
        };
        assert_eq!(
            response.header("location"),
            Some("https://example.test/poll")
        );
    }

    #[test]
    fn parses_backend_health_response_into_strong_type() -> eyre::Result<()> {
        let response = SerializableRestResponse {
            status: 200,
            ok: true,
            reason_phrase: Some("OK".to_string()),
            headers: BTreeMap::new(),
            body: RestResponseBody::Json(serde_json::json!({
                "backendAddressPools": [
                    {
                        "backendAddressPool": {
                            "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/applicationGateways/my-agw/backendAddressPools/pool-a"
                        },
                        "backendHttpSettingsCollection": [
                            {
                                "backendHttpSettings": {
                                    "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/applicationGateways/my-agw/backendHttpSettingsCollection/settings-a"
                                },
                                "servers": [
                                    {
                                        "address": "10.0.0.5",
                                        "health": "Healthy",
                                        "healthProbeLog": "Success",
                                        "healthProbeErrorName": "SuccessWithStatusCode",
                                        "backendCertificateChainMetadata": {
                                            "certificateChainMetadata": []
                                        }
                                    }
                                ]
                            }
                        ]
                    }
                ]
            })),
        };

        let parsed: AzureApplicationGatewayResourceBackendHealthResponse =
            parse_backend_health_response(response)?;
        assert_eq!(parsed.backend_address_pools.len(), 1);
        assert_eq!(
            parsed.backend_address_pools[0].backend_http_settings_collection[0].servers[0].health,
            AzureApplicationGatewayResourceBackendHealthServerHealth::Healthy
        );
        assert_eq!(
            parsed.backend_address_pools[0].backend_http_settings_collection[0].servers[0]
                .health_probe_error_name,
            Some(AzureApplicationGatewayResourceBackendHealthProbeErrorName::SuccessWithStatusCode)
        );
        Ok(())
    }
}
