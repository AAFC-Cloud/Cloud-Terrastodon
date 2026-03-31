use crate::AzureApplicationGatewayResourceReference;
use crate::serde_helpers::deserialize_default_if_null;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResourceBackendHealthResponse {
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub backend_address_pools: Vec<AzureApplicationGatewayResourceBackendHealthPool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResourceBackendHealthPool {
    pub backend_address_pool: AzureApplicationGatewayResourceReference,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub backend_http_settings_collection:
        Vec<AzureApplicationGatewayResourceBackendHealthHttpSettings>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResourceBackendHealthHttpSettings {
    pub backend_http_settings: AzureApplicationGatewayResourceReference,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub servers: Vec<AzureApplicationGatewayResourceBackendHealthServer>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResourceBackendHealthServer {
    pub address: String,
    pub health: AzureApplicationGatewayResourceBackendHealthServerHealth,
    #[serde(default)]
    pub health_probe_log: Option<String>,
    #[serde(default)]
    pub health_probe_error_name: Option<AzureApplicationGatewayResourceBackendHealthProbeErrorName>,
    #[serde(default)]
    pub ip_configuration: Option<AzureApplicationGatewayResourceReference>,
    #[serde(default)]
    pub backend_certificate_chain_metadata:
        Option<AzureApplicationGatewayResourceBackendHealthCertificateChainMetadata>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

impl Serialize for AzureApplicationGatewayResourceBackendHealthServerHealth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = String::from(self.clone());
        serializer.serialize_str(&value)
    }
}

impl<'de> Deserialize<'de> for AzureApplicationGatewayResourceBackendHealthServerHealth {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from(String::deserialize(deserializer)?))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AzureApplicationGatewayResourceBackendHealthProbeErrorName {
    SuccessWithStatusCode,
    HttpStatusCodeMismatchWithStatusCode,
    Other(String),
}

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

impl Serialize for AzureApplicationGatewayResourceBackendHealthProbeErrorName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = String::from(self.clone());
        serializer.serialize_str(&value)
    }
}

impl<'de> Deserialize<'de> for AzureApplicationGatewayResourceBackendHealthProbeErrorName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from(String::deserialize(deserializer)?))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResourceBackendHealthCertificateChainMetadata {
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub certificate_chain_metadata:
        Vec<AzureApplicationGatewayResourceBackendHealthCertificateMetadataEntry>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct AzureApplicationGatewayResourceBackendHealthCertificateMetadataEntry {
    #[serde(flatten)]
    pub fields: BTreeMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::AzureApplicationGatewayResourceBackendHealthProbeErrorName;
    use super::AzureApplicationGatewayResourceBackendHealthResponse;
    use super::AzureApplicationGatewayResourceBackendHealthServerHealth;

    #[test]
    fn deserializes_backend_health_response() -> eyre::Result<()> {
        let response = serde_json::from_str::<AzureApplicationGatewayResourceBackendHealthResponse>(
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
        assert_eq!(
          server.health_probe_error_name,
          Some(
            AzureApplicationGatewayResourceBackendHealthProbeErrorName::HttpStatusCodeMismatchWithStatusCode,
          )
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
        let response = serde_json::from_str::<AzureApplicationGatewayResourceBackendHealthResponse>(
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
        assert_eq!(
            server.health_probe_error_name,
            Some(
                AzureApplicationGatewayResourceBackendHealthProbeErrorName::Other(
                    "BrandNewErrorName".to_string()
                )
            )
        );

        Ok(())
    }
}
