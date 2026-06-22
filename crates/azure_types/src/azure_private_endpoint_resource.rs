use crate::AzureLocationName;
use crate::AzureNetworkInterfaceResourceId;
use crate::AzureNetworkInterfaceResourceName;
use crate::AzurePrivateEndpointResourceId;
use crate::AzurePrivateEndpointResourceName;
use crate::AzureTenantId;
use crate::SubnetId;
use facet_json::RawJson;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePrivateEndpointResource {
    pub id: AzurePrivateEndpointResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzurePrivateEndpointResourceName,
    pub location: AzureLocationName,
    #[facet(default)]
    pub tags: HashMap<String, String>,
    pub properties: AzurePrivateEndpointResourceProperties,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePrivateEndpointResourceProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub resource_guid: Option<String>,
    #[facet(default)]
    pub ip_configurations: Vec<RawJson<'static>>,
    #[facet(default)]
    pub network_interfaces: Vec<AzurePrivateEndpointNetworkInterfaceReference>,
    #[facet(default)]
    pub subnet: Option<AzurePrivateEndpointSubnetReference>,
    #[facet(default)]
    pub manual_private_link_service_connections:
        Vec<AzurePrivateEndpointPrivateLinkServiceConnection>,
    #[facet(default)]
    pub private_link_service_connections: Vec<AzurePrivateEndpointPrivateLinkServiceConnection>,
    #[facet(default)]
    pub custom_network_interface_name: Option<AzureNetworkInterfaceResourceName>,
    #[facet(default)]
    pub custom_dns_configs: Vec<AzurePrivateEndpointCustomDnsConfig>,
    #[facet(default)]
    pub ip_version_type: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
pub struct AzurePrivateEndpointNetworkInterfaceReference {
    pub id: AzureNetworkInterfaceResourceId,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
pub struct AzurePrivateEndpointSubnetReference {
    pub id: SubnetId,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
pub struct AzurePrivateEndpointPrivateLinkServiceConnection {
    pub name: String,
    pub id: String,
    #[facet(default)]
    pub etag: Option<String>,
    #[facet(rename = "type", default)]
    pub resource_type: Option<String>,
    pub properties: AzurePrivateEndpointPrivateLinkServiceConnectionProperties,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePrivateEndpointPrivateLinkServiceConnectionProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub private_link_service_connection_state:
        Option<AzurePrivateEndpointPrivateLinkServiceConnectionState>,
    #[facet(default)]
    pub private_link_service_id: Option<String>,
    #[facet(default)]
    pub group_ids: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePrivateEndpointPrivateLinkServiceConnectionState {
    #[facet(default)]
    pub description: Option<String>,
    #[facet(default)]
    pub status: Option<String>,
    #[facet(default)]
    pub actions_required: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePrivateEndpointCustomDnsConfig {
    #[facet(default, opaque, proxy = crate::IpAddrVecProxy)]
    pub ip_addresses: Vec<IpAddr>,
    #[facet(default)]
    pub fqdn: Option<String>,
}
