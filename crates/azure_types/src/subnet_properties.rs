use crate::AddressPrefixes;
use crate::RouteTableId;

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct SubnetProperties {
    #[facet(flatten, opaque, proxy = crate::address_prefixes::AddressPrefixesProxy)]
    pub address_prefixes: AddressPrefixes,

    #[facet(rename = "networkSecurityGroup")]
    pub network_security_group: Option<NetworkSecurityGroupReference>,

    #[facet(rename = "routeTable")]
    pub route_table: Option<RouteTableReference>,

    #[facet(rename = "privateEndpointNetworkPolicies")]
    pub private_endpoint_network_policies: String,

    #[facet(rename = "privateLinkServiceNetworkPolicies")]
    pub private_link_service_network_policies: String,

    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<Delegation>)]
    pub delegations: Vec<Delegation>,

    #[facet(
        rename = "serviceEndpoints",
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<ServiceEndpoint>
    )]
    pub service_endpoints: Vec<ServiceEndpoint>,

    #[facet(
        rename = "serviceEndpointPolicies",
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<ServiceEndpointPolicyReference>
    )]
    pub service_endpoint_policies: Vec<ServiceEndpointPolicyReference>,

    #[facet(rename = "natGateway")]
    pub nat_gateway: Option<NatGatewayReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct NetworkSecurityGroupReference {
    pub id: String,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct RouteTableReference {
    pub id: RouteTableId,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct Delegation {
    pub name: String,
    pub properties: DelegationProperties,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct DelegationProperties {
    #[facet(rename = "serviceName")]
    pub service_name: String,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<String>)]
    pub actions: Vec<String>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct ServiceEndpoint {
    pub service: String,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<String>)]
    pub locations: Vec<String>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct ServiceEndpointPolicyReference {
    pub id: String,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct NatGatewayReference {
    pub id: String,
}
