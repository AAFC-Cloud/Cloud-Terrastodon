use crate::prelude::Subnet;
use crate::prelude::VirtualNetworkId;
use crate::prelude::VirtualNetworkPeeringId;
use crate::prelude::VirtualNetworkPeeringName;
use ipnetwork::Ipv4Network;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VirtualNetworkProperties {
    pub address_space: VirtualNetworkAddressSpace,
    pub subnets: Vec<Subnet>,
    pub virtual_network_peerings: Vec<VirtualNetworkPeering>,
    pub resource_guid: String,
    pub provisioning_state: String,
    pub enable_ddos_protection: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VirtualNetworkAddressSpace {
    #[serde(rename = "addressPrefixes")]
    pub address_prefixes: Vec<Ipv4Network>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VirtualNetworkPeering {
    pub id: VirtualNetworkPeeringId,
    pub name: VirtualNetworkPeeringName,
    pub properties: VirtualNetworkPeeringProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VirtualNetworkPeeringProperties {
    #[serde(rename = "allowVirtualNetworkAccess")]
    pub allow_virtual_network_access: bool,

    #[serde(rename = "allowForwardedTraffic")]
    pub allow_forwarded_traffic: bool,

    #[serde(rename = "allowGatewayTransit")]
    pub allow_gateway_transit: bool,

    #[serde(rename = "useRemoteGateways")]
    pub use_remote_gateways: bool,

    #[serde(rename = "remoteVirtualNetwork")]
    pub remote_virtual_network: RemoteVirtualNetworkReference,

    #[serde(rename = "peeringState")]
    pub peering_state: String,

    #[serde(rename = "provisioningState")]
    pub provisioning_state: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RemoteVirtualNetworkReference {
    pub id: VirtualNetworkId,
}
