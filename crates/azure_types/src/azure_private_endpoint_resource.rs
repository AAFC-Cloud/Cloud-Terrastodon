use crate::AzureLocationName;
use crate::AzureNetworkInterfaceResourceId;
use crate::AzureNetworkInterfaceResourceName;
use crate::AzurePrivateEndpointResourceId;
use crate::AzurePrivateEndpointResourceName;
use crate::AzureTenantId;
use crate::SubnetId;
use crate::serde_helpers::deserialize_default_if_null;
use crate::serde_helpers::deserialize_none_if_empty_string;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePrivateEndpointResource {
    pub id: AzurePrivateEndpointResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzurePrivateEndpointResourceName,
    pub location: AzureLocationName,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
    pub properties: AzurePrivateEndpointResourceProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePrivateEndpointResourceProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub resource_guid: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub ip_configurations: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub network_interfaces: Vec<AzurePrivateEndpointNetworkInterfaceReference>,
    #[serde(default)]
    pub subnet: Option<AzurePrivateEndpointSubnetReference>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub manual_private_link_service_connections:
        Vec<AzurePrivateEndpointPrivateLinkServiceConnection>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub private_link_service_connections: Vec<AzurePrivateEndpointPrivateLinkServiceConnection>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub custom_network_interface_name: Option<AzureNetworkInterfaceResourceName>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub custom_dns_configs: Vec<AzurePrivateEndpointCustomDnsConfig>,
    #[serde(default)]
    pub ip_version_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzurePrivateEndpointNetworkInterfaceReference {
    pub id: AzureNetworkInterfaceResourceId,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzurePrivateEndpointSubnetReference {
    pub id: SubnetId,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzurePrivateEndpointPrivateLinkServiceConnection {
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(rename = "type")]
    #[serde(default)]
    pub resource_type: Option<String>,
    pub properties: AzurePrivateEndpointPrivateLinkServiceConnectionProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePrivateEndpointPrivateLinkServiceConnectionProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub private_link_service_connection_state:
        Option<AzurePrivateEndpointPrivateLinkServiceConnectionState>,
    #[serde(default)]
    pub private_link_service_id: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub group_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePrivateEndpointPrivateLinkServiceConnectionState {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub actions_required: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePrivateEndpointCustomDnsConfig {
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub ip_addresses: Vec<IpAddr>,
    #[serde(default)]
    pub fqdn: Option<String>,
}
