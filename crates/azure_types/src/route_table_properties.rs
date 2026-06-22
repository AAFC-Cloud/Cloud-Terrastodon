use crate::AddressPrefix;

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct RouteTableProperties {
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<Route>)]
    pub routes: Vec<Route>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<SubnetReference>)]
    pub subnets: Vec<SubnetReference>,
    pub resource_guid: String,
    pub provisioning_state: String,
    #[facet(default)]
    pub disable_bgp_route_propagation: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct Route {
    pub id: String,
    pub name: String,
    #[facet(rename = "type")]
    pub resource_type: String,
    pub etag: String,
    pub properties: RouteProperties,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct RouteProperties {
    pub address_prefix: AddressPrefix,
    pub next_hop_type: NextHopType,
    pub next_hop_ip_address: Option<String>,
    pub provisioning_state: String,
    pub has_bgp_override: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[repr(C)]
pub enum NextHopType {
    #[facet(rename = "VirtualNetworkGateway")]
    VirtualNetworkGateway,
    #[facet(rename = "VnetLocal")]
    VnetLocal,
    #[facet(rename = "Internet")]
    Internet,
    #[facet(rename = "VirtualAppliance")]
    VirtualAppliance,
    #[facet(rename = "None")]
    None,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct SubnetReference {
    #[facet(default)]
    pub id: Option<String>,
}
