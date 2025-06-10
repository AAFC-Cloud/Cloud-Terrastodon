use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SubnetProperties {
    #[serde(rename = "addressPrefix")]
    pub address_prefix: Option<String>,
    
    #[serde(rename = "networkSecurityGroup")]
    pub network_security_group: Option<NetworkSecurityGroupReference>,
    
    #[serde(rename = "routeTable")]
    pub route_table: Option<RouteTableReference>,
    
    #[serde(rename = "privateEndpointNetworkPolicies")]
    pub private_endpoint_network_policies: Option<String>,
    
    #[serde(rename = "privateLinkServiceNetworkPolicies")]
    pub private_link_service_network_policies: Option<String>,
    
    pub delegations: Option<Vec<Delegation>>,
    
    #[serde(rename = "serviceEndpoints")]
    pub service_endpoints: Option<Vec<ServiceEndpoint>>,
    
    #[serde(rename = "serviceEndpointPolicies")]
    pub service_endpoint_policies: Option<Vec<ServiceEndpointPolicyReference>>,
    
    #[serde(rename = "natGateway")]
    pub nat_gateway: Option<NatGatewayReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NetworkSecurityGroupReference {
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RouteTableReference {
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Delegation {
    pub name: Option<String>,
    pub properties: Option<DelegationProperties>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DelegationProperties {
    #[serde(rename = "serviceName")]
    pub service_name: Option<String>,
    pub actions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ServiceEndpoint {
    pub service: Option<String>,
    pub locations: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ServiceEndpointPolicyReference {
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NatGatewayReference {
    pub id: Option<String>,
}
