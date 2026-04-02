use crate::AzureAppServiceResourceId;
use crate::AzureAppServiceResourceName;
use crate::AzureLocationName;
use crate::AzureTenantId;
use crate::serde_helpers::deserialize_default_if_null;
use arbitrary::Arbitrary;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Arbitrary)]
#[serde(rename_all = "camelCase")]
pub struct AzureAppServiceResource {
    pub id: AzureAppServiceResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzureAppServiceResourceName,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_app_service_resource_kinds")]
    pub kind: Vec<AzureAppServiceResourceKind>,
    pub location: AzureLocationName,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
    pub properties: AzureAppServiceResourceProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Arbitrary)]
pub enum AzureAppServiceResourceKind {
    #[serde(rename = "workflowapp")]
    WorkflowApp,
    #[serde(rename = "azurecontainerapps")]
    AzureContainerApps,
    #[serde(rename = "container")]
    Container,
    #[serde(rename = "app")]
    App,
    #[serde(rename = "functionapp")]
    FunctionApp,
    #[serde(rename = "linux")]
    Linux,
    #[serde(untagged)]
    Other(String),
}

fn deserialize_app_service_resource_kinds<'de, D>(
    deserializer: D,
) -> Result<Vec<AzureAppServiceResourceKind>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };

    raw.split(',')
        .map(str::trim)
        .filter(|kind| !kind.is_empty())
        .map(|kind| {
            serde_json::from_value::<AzureAppServiceResourceKind>(serde_json::Value::String(
                kind.to_string(),
            ))
            .map_err(serde::de::Error::custom)
        })
        .collect()
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Arbitrary)]
#[serde(rename_all = "camelCase")]
pub struct AzureAppServiceResourceProperties {
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub public_network_access: Option<String>,
    #[serde(default)]
    pub default_host_name: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub host_names: Vec<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub enabled_host_names: Vec<String>,
    #[serde(default)]
    pub inbound_ip_address: Option<IpAddr>,
    #[serde(default)]
    pub inbound_ipv6_address: Option<IpAddr>,
    #[serde(default)]
    pub outbound_ip_addresses: Option<String>,
    #[serde(default)]
    pub possible_outbound_ip_addresses: Option<String>,
    #[serde(default)]
    pub possible_outbound_ipv6_addresses: Option<String>,
    #[serde(default)]
    pub possible_inbound_ip_addresses: Option<String>,
    #[serde(default)]
    pub possible_inbound_ipv6_addresses: Option<String>,
    #[serde(default)]
    pub private_link_identifiers: Option<String>,
    #[serde(default)]
    pub server_farm_id: Option<String>,
    #[serde(default)]
    pub virtual_network_subnet_id: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub private_endpoint_connections: Vec<AzureAppServicePrivateEndpointConnection>,
    #[serde(default)]
    pub site_config: Option<AzureAppServiceSiteConfig>,
    #[serde(flatten)]
    #[arbitrary(default)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Arbitrary)]
#[serde(rename_all = "camelCase")]
pub struct AzureAppServicePrivateEndpointConnection {
    pub name: String,
    #[serde(rename = "type")]
    #[serde(default)]
    pub resource_type: Option<String>,
    pub id: String,
    #[serde(default)]
    pub location: Option<AzureLocationName>,
    pub properties: AzureAppServicePrivateEndpointConnectionProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Arbitrary)]
#[serde(rename_all = "camelCase")]
pub struct AzureAppServicePrivateEndpointConnectionProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub private_link_service_connection_state:
        Option<AzureAppServicePrivateLinkServiceConnectionState>,
    #[serde(default)]
    pub private_endpoint: Option<AzureAppServiceResourceReference>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub ip_addresses: Vec<IpAddr>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub group_ids: Vec<String>,
    #[serde(flatten)]
    #[arbitrary(default)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Arbitrary)]
#[serde(rename_all = "camelCase")]
pub struct AzureAppServicePrivateLinkServiceConnectionState {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub actions_required: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Arbitrary)]
pub struct AzureAppServiceResourceReference {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Arbitrary)]
#[serde(rename_all = "camelCase")]
pub struct AzureAppServiceSiteConfig {
    #[serde(default)]
    pub public_network_access: Option<String>,
    #[serde(default)]
    pub linux_fx_version: Option<String>,
    #[serde(default)]
    pub windows_fx_version: Option<String>,
    #[serde(default)]
    pub always_on: Option<bool>,
    #[serde(flatten)]
    #[arbitrary(default)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::AzureAppServiceResourceKind;

    #[test]
    fn deserializes_unknown_app_service_kind_to_other() -> eyre::Result<()> {
        let kind = serde_json::from_str::<AzureAppServiceResourceKind>("\"brandnewkind\"")?;

        assert_eq!(
            kind,
            AzureAppServiceResourceKind::Other("brandnewkind".to_string())
        );

        Ok(())
    }
}
