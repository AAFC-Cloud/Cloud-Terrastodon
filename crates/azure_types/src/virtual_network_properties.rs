use crate::prelude::Subnet;
use ipnetwork::Ipv4Network;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VirtualNetworkProperties {
    pub address_space: AddressSpace,
    pub subnets: Vec<Subnet>,
    pub virtual_network_peerings: Vec<VirtualNetworkPeering>,
    pub resource_guid: String,
    pub provisioning_state: String,
    pub enable_ddos_protection: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AddressSpace {
    #[serde(rename = "addressPrefixes")]
    pub address_prefixes: Vec<Ipv4Network>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VirtualNetworkPeering {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub etag: Option<String>,
    pub properties: Option<VirtualNetworkPeeringProperties>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VirtualNetworkPeeringProperties {
    #[serde(rename = "allowVirtualNetworkAccess")]
    pub allow_virtual_network_access: Option<bool>,

    #[serde(rename = "allowForwardedTraffic")]
    pub allow_forwarded_traffic: Option<bool>,

    #[serde(rename = "allowGatewayTransit")]
    pub allow_gateway_transit: Option<bool>,

    #[serde(rename = "useRemoteGateways")]
    pub use_remote_gateways: Option<bool>,

    #[serde(rename = "remoteVirtualNetwork")]
    pub remote_virtual_network: Option<RemoteVirtualNetworkReference>,

    #[serde(rename = "peeringState")]
    pub peering_state: Option<String>,

    #[serde(rename = "provisioningState")]
    pub provisioning_state: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RemoteVirtualNetworkReference {
    pub id: Option<String>,
}
