use ipnetwork::Ipv4Network;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RouteTableProperties {
    pub routes: Vec<Route>,
    #[serde(default = "Vec::new")]
    pub subnets: Vec<SubnetReference>,
    pub resource_guid: String,
    pub provisioning_state: String,
    pub disable_bgp_route_propagation: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    pub etag: String,
    pub properties: RouteProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RouteProperties {
    pub address_prefix: Ipv4Network,
    pub next_hop_type: NextHopType,
    pub next_hop_ip_address: Option<String>,
    pub provisioning_state: String,
    pub has_bgp_override: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum NextHopType {
    #[serde(rename = "VirtualNetworkGateway")]
    VirtualNetworkGateway,
    #[serde(rename = "VnetLocal")]
    VnetLocal,
    #[serde(rename = "Internet")]
    Internet,
    #[serde(rename = "VirtualAppliance")]
    VirtualAppliance,
    #[serde(rename = "None")]
    None,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SubnetReference {
    pub id: Option<String>,
}
