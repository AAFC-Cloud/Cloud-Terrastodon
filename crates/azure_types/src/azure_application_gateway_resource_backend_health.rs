use crate::AzureApplicationGatewayResourceReference;
use facet_json::RawJson;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResourceBackendHealthResponse {
    #[facet(default)]
    pub backend_address_pools: Vec<AzureApplicationGatewayResourceBackendHealthPool>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResourceBackendHealthPool {
    pub backend_address_pool: AzureApplicationGatewayResourceReference,
    #[facet(default)]
    pub backend_http_settings_collection:
        Vec<AzureApplicationGatewayResourceBackendHealthHttpSettings>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResourceBackendHealthHttpSettings {
    pub backend_http_settings: AzureApplicationGatewayResourceReference,
    #[facet(default)]
    pub servers: Vec<AzureApplicationGatewayResourceBackendHealthServer>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResourceBackendHealthServer {
    pub address: String,
    pub health: AzureApplicationGatewayResourceBackendHealthServerHealth,
    #[facet(default)]
    pub health_probe_log: Option<String>,
    #[facet(default)]
    pub health_probe_error_name: Option<AzureApplicationGatewayResourceBackendHealthProbeErrorName>,
    #[facet(default)]
    pub ip_configuration: Option<AzureApplicationGatewayResourceReference>,
    #[facet(default)]
    pub backend_certificate_chain_metadata:
        Option<AzureApplicationGatewayResourceBackendHealthCertificateChainMetadata>,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureApplicationGatewayResourceBackendHealthServerHealth {
    Unknown,
    Up,
    Down,
    Partial,
    Draining,
    Healthy,
    Unhealthy,
    Other(String),
}
crate::impl_facet_string_proxy_serialize!(
    AzureApplicationGatewayResourceBackendHealthServerHealth,
    value => String::from(value.clone())
);

impl From<String> for AzureApplicationGatewayResourceBackendHealthServerHealth {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Unknown" => Self::Unknown,
            "Up" => Self::Up,
            "Down" => Self::Down,
            "Partial" => Self::Partial,
            "Draining" => Self::Draining,
            "Healthy" => Self::Healthy,
            "Unhealthy" => Self::Unhealthy,
            _ => Self::Other(value),
        }
    }
}

impl From<AzureApplicationGatewayResourceBackendHealthServerHealth> for String {
    fn from(value: AzureApplicationGatewayResourceBackendHealthServerHealth) -> Self {
        match value {
            AzureApplicationGatewayResourceBackendHealthServerHealth::Unknown => {
                "Unknown".to_string()
            }
            AzureApplicationGatewayResourceBackendHealthServerHealth::Up => "Up".to_string(),
            AzureApplicationGatewayResourceBackendHealthServerHealth::Down => "Down".to_string(),
            AzureApplicationGatewayResourceBackendHealthServerHealth::Partial => {
                "Partial".to_string()
            }
            AzureApplicationGatewayResourceBackendHealthServerHealth::Draining => {
                "Draining".to_string()
            }
            AzureApplicationGatewayResourceBackendHealthServerHealth::Healthy => {
                "Healthy".to_string()
            }
            AzureApplicationGatewayResourceBackendHealthServerHealth::Unhealthy => {
                "Unhealthy".to_string()
            }
            AzureApplicationGatewayResourceBackendHealthServerHealth::Other(value) => value,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureApplicationGatewayResourceBackendHealthProbeErrorName {
    SuccessWithStatusCode,
    HttpStatusCodeMismatchWithStatusCode,
    Other(String),
}
crate::impl_facet_string_proxy_serialize!(
    AzureApplicationGatewayResourceBackendHealthProbeErrorName,
    value => String::from(value.clone())
);

impl From<String> for AzureApplicationGatewayResourceBackendHealthProbeErrorName {
    fn from(value: String) -> Self {
        match value.as_str() {
            "SuccessWithStatusCode" => Self::SuccessWithStatusCode,
            "HttpStatusCodeMismatchWithStatusCode" => Self::HttpStatusCodeMismatchWithStatusCode,
            _ => Self::Other(value),
        }
    }
}

impl From<AzureApplicationGatewayResourceBackendHealthProbeErrorName> for String {
    fn from(value: AzureApplicationGatewayResourceBackendHealthProbeErrorName) -> Self {
        match value {
      AzureApplicationGatewayResourceBackendHealthProbeErrorName::SuccessWithStatusCode => {
        "SuccessWithStatusCode".to_string()
      }
      AzureApplicationGatewayResourceBackendHealthProbeErrorName::HttpStatusCodeMismatchWithStatusCode => {
        "HttpStatusCodeMismatchWithStatusCode".to_string()
      }
      AzureApplicationGatewayResourceBackendHealthProbeErrorName::Other(value) => value,
    }
    }
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResourceBackendHealthCertificateChainMetadata {
    #[facet(default)]
    pub certificate_chain_metadata:
        Vec<AzureApplicationGatewayResourceBackendHealthCertificateMetadataEntry>,
}

#[derive(Debug, PartialEq, Clone, Default, facet::Facet)]
pub struct AzureApplicationGatewayResourceBackendHealthCertificateMetadataEntry {
    #[facet(flatten)]
    pub fields: BTreeMap<String, RawJson<'static>>,
}

#[cfg(test)]
mod tests {
    use super::AzureApplicationGatewayResourceBackendHealthProbeErrorName;
    use super::AzureApplicationGatewayResourceBackendHealthResponse;
    use super::AzureApplicationGatewayResourceBackendHealthServerHealth;

    #[test]
    fn deserializes_backend_health_response() -> eyre::Result<()> {
        let response = facet_json::from_str::<AzureApplicationGatewayResourceBackendHealthResponse>(
            r#"
            {
              "backendAddressPools": [
                {
                  "backendAddressPool": {
                    "id": "/subscriptions/9f8ec489-719d-4f43-821b-ae1f712bc722/resourceGroups/AGPcCKAN-UAT-rg/providers/Microsoft.Network/applicationGateways/AGPcCKAN-Uat-WAF-AAFC/backendAddressPools/terra-frontend-addr-pool"
                  },
                  "backendHttpSettingsCollection": [
                    {
                      "backendHttpSettings": {
                        "id": "/subscriptions/9f8ec489-719d-4f43-821b-ae1f712bc722/resourceGroups/AGPcCKAN-UAT-rg/providers/Microsoft.Network/applicationGateways/AGPcCKAN-Uat-WAF-AAFC/backendHttpSettingsCollection/terra-frontend-conf-https"
                      },
                      "servers": [
                        {
                          "address": "10.0.0.123",
                          "health": "Unhealthy",
                          "healthProbeLog": "Received invalid status code: 502 in the backend server's HTTP response.",
                          "healthProbeErrorName": "HttpStatusCodeMismatchWithStatusCode",
                          "backendCertificateChainMetadata": {
                            "certificateChainMetadata": []
                          }
                        }
                      ]
                    }
                  ]
                }
              ]
            }
            "#,
        )?;

        assert_eq!(response.backend_address_pools.len(), 1);
        let pool = &response.backend_address_pools[0];
        assert_eq!(
            pool.backend_address_pool.id,
            "/subscriptions/9f8ec489-719d-4f43-821b-ae1f712bc722/resourceGroups/AGPcCKAN-UAT-rg/providers/Microsoft.Network/applicationGateways/AGPcCKAN-Uat-WAF-AAFC/backendAddressPools/terra-frontend-addr-pool"
        );
        assert_eq!(pool.backend_http_settings_collection.len(), 1);
        let settings = &pool.backend_http_settings_collection[0];
        assert_eq!(settings.servers.len(), 1);
        let server = &settings.servers[0];
        assert_eq!(server.address, "10.0.0.123");
        assert_eq!(
            server.health,
            AzureApplicationGatewayResourceBackendHealthServerHealth::Unhealthy
        );
        assert_eq!(facet_json::to_string(&server.health)?, "\"Unhealthy\"");
        assert_eq!(
          server.health_probe_error_name,
          Some(
            AzureApplicationGatewayResourceBackendHealthProbeErrorName::HttpStatusCodeMismatchWithStatusCode,
          )
        );
        assert_eq!(
            facet_json::to_string(&server.health_probe_error_name)?,
            "\"HttpStatusCodeMismatchWithStatusCode\""
        );
        assert_eq!(
            server
                .backend_certificate_chain_metadata
                .as_ref()
                .map(|metadata| metadata.certificate_chain_metadata.len()),
            Some(0)
        );

        Ok(())
    }

    #[test]
    fn preserves_unknown_enum_values() -> eyre::Result<()> {
        let response = facet_json::from_str::<AzureApplicationGatewayResourceBackendHealthResponse>(
            r#"
            {
              "backendAddressPools": [
                {
                  "backendAddressPool": {
                    "id": "/subscriptions/x/resourceGroups/rg/providers/Microsoft.Network/applicationGateways/agw/backendAddressPools/pool"
                  },
                  "backendHttpSettingsCollection": [
                    {
                      "backendHttpSettings": {
                        "id": "/subscriptions/x/resourceGroups/rg/providers/Microsoft.Network/applicationGateways/agw/backendHttpSettingsCollection/settings"
                      },
                      "servers": [
                        {
                          "address": "example.internal",
                          "health": "Wobbly",
                          "healthProbeErrorName": "BrandNewErrorName"
                        }
                      ]
                    }
                  ]
                }
              ]
            }
            "#,
        )?;

        let server =
            &response.backend_address_pools[0].backend_http_settings_collection[0].servers[0];
        assert_eq!(
            server.health,
            AzureApplicationGatewayResourceBackendHealthServerHealth::Other("Wobbly".to_string())
        );
        assert_eq!(facet_json::to_string(&server.health)?, "\"Wobbly\"");
        assert_eq!(
            server.health_probe_error_name,
            Some(
                AzureApplicationGatewayResourceBackendHealthProbeErrorName::Other(
                    "BrandNewErrorName".to_string()
                )
            )
        );
        assert_eq!(
            facet_json::to_string(&server.health_probe_error_name)?,
            "\"BrandNewErrorName\""
        );

        Ok(())
    }
}
