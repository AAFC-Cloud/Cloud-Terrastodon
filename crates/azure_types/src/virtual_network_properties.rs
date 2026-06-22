use crate::Subnet;
use crate::VirtualNetworkId;
use crate::VirtualNetworkPeeringId;
use crate::VirtualNetworkPeeringName;
use ipnetwork::Ipv4Network;

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct VirtualNetworkProperties {
    pub address_space: VirtualNetworkAddressSpace,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<Subnet>)]
    pub subnets: Vec<Subnet>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<VirtualNetworkPeering>
    )]
    pub virtual_network_peerings: Vec<VirtualNetworkPeering>,
    pub resource_guid: String,
    pub provisioning_state: String,
    pub enable_ddos_protection: bool,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct VirtualNetworkAddressSpace {
    #[facet(
        rename = "addressPrefixes",
        opaque,
        proxy = crate::Ipv4NetworkVecProxy
    )]
    pub address_prefixes: Vec<Ipv4Network>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct VirtualNetworkPeering {
    pub id: VirtualNetworkPeeringId,
    pub name: VirtualNetworkPeeringName,
    pub properties: VirtualNetworkPeeringProperties,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct VirtualNetworkPeeringProperties {
    #[facet(rename = "allowVirtualNetworkAccess")]
    pub allow_virtual_network_access: bool,

    #[facet(rename = "allowForwardedTraffic")]
    pub allow_forwarded_traffic: bool,

    #[facet(rename = "allowGatewayTransit")]
    pub allow_gateway_transit: bool,

    #[facet(rename = "useRemoteGateways")]
    pub use_remote_gateways: bool,

    #[facet(rename = "remoteVirtualNetwork")]
    pub remote_virtual_network: RemoteVirtualNetworkReference,

    #[facet(rename = "peeringState")]
    pub peering_state: String,

    #[facet(rename = "provisioningState")]
    pub provisioning_state: String,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct RemoteVirtualNetworkReference {
    pub id: VirtualNetworkId,
}
