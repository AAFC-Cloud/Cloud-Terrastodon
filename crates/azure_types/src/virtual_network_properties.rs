use crate::Subnet;
use crate::VirtualNetworkId;
use crate::VirtualNetworkPeeringId;
use crate::VirtualNetworkPeeringName;
use arbitrary::Arbitrary;
use ipnetwork::Ipv4Network;

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct VirtualNetworkProperties {
    pub address_space: VirtualNetworkAddressSpace,
    #[facet(default, proxy = crate::VecDefaultNullProxy<Subnet>)]
    pub subnets: Vec<Subnet>,
    #[facet(
        default,
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

impl<'a> Arbitrary<'a> for VirtualNetworkAddressSpace {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let count = u.int_in_range(1..=3)?;
        let mut address_prefixes = Vec::with_capacity(count);
        for index in 0..count {
            let network = Ipv4Network::new(std::net::Ipv4Addr::new(10, index as u8, 0, 0), 24)
                .map_err(|_| arbitrary::Error::IncorrectFormat)?;
            address_prefixes.push(network);
        }
        Ok(Self { address_prefixes })
    }
}

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
pub struct VirtualNetworkPeering {
    pub id: VirtualNetworkPeeringId,
    pub name: VirtualNetworkPeeringName,
    pub properties: VirtualNetworkPeeringProperties,
}

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
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

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
pub struct RemoteVirtualNetworkReference {
    pub id: VirtualNetworkId,
}
