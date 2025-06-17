use crate::prelude::AddressPrefixes;
use crate::prelude::RouteTable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SubnetProperties {
    #[serde(flatten)]
    pub address_prefixes: AddressPrefixes,

    #[serde(rename = "networkSecurityGroup")]
    pub network_security_group: Option<NetworkSecurityGroupReference>,

    #[serde(rename = "routeTable")]
    pub route_table: Option<RouteTableReference>,

    #[serde(rename = "privateEndpointNetworkPolicies")]
    pub private_endpoint_network_policies: String,

    #[serde(rename = "privateLinkServiceNetworkPolicies")]
    pub private_link_service_network_policies: String,

    pub delegations: Vec<Delegation>,

    #[serde(rename = "serviceEndpoints")]
    #[serde(default)]
    pub service_endpoints: Vec<ServiceEndpoint>,

    #[serde(rename = "serviceEndpointPolicies")]
    #[serde(default)]
    pub service_endpoint_policies: Vec<ServiceEndpointPolicyReference>,

    #[serde(rename = "natGateway")]
    pub nat_gateway: Option<NatGatewayReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NetworkSecurityGroupReference {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RouteTableReference {
    pub id: RouteTable,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Delegation {
    pub name: String,
    pub properties: DelegationProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DelegationProperties {
    #[serde(rename = "serviceName")]
    pub service_name: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ServiceEndpoint {
    pub service: String,
    pub locations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ServiceEndpointPolicyReference {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NatGatewayReference {
    pub id: String,
}
