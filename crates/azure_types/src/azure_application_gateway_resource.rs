use crate::AzureApplicationGatewayResourceId;
use crate::AzureApplicationGatewayResourceName;
use crate::AzureLocationName;
use crate::AzureTenantId;
use crate::serde_helpers::deserialize_default_if_null;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResource {
    pub id: AzureApplicationGatewayResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzureApplicationGatewayResourceName,
    pub location: AzureLocationName,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
    #[serde(default)]
    pub identity: Option<AzureApplicationGatewayIdentity>,
    pub properties: AzureApplicationGatewayResourceProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayIdentity {
    #[serde(rename = "type")]
    pub identity_type: String,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub user_assigned_identities: HashMap<String, AzureApplicationGatewayUserAssignedIdentity>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayUserAssignedIdentity {
    #[serde(default)]
    pub principal_id: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResourceProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub resource_guid: Option<String>,
    #[serde(default)]
    pub sku: Option<AzureApplicationGatewaySku>,
    #[serde(default)]
    pub operational_state: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub gateway_ip_configurations: Vec<
        AzureApplicationGatewaySubResource<AzureApplicationGatewayGatewayIpConfigurationProperties>,
    >,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub ssl_certificates:
        Vec<AzureApplicationGatewaySubResource<AzureApplicationGatewaySslCertificateProperties>>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub trusted_root_certificates: Vec<
        AzureApplicationGatewaySubResource<AzureApplicationGatewayTrustedRootCertificateProperties>,
    >,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub trusted_client_certificates: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub ssl_profiles: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub frontend_ip_configurations: Vec<
        AzureApplicationGatewaySubResource<
            AzureApplicationGatewayFrontendIpConfigurationProperties,
        >,
    >,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub frontend_ports:
        Vec<AzureApplicationGatewaySubResource<AzureApplicationGatewayFrontendPortProperties>>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub backend_address_pools: Vec<
        AzureApplicationGatewaySubResource<AzureApplicationGatewayBackendAddressPoolProperties>,
    >,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub load_distribution_policies: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub backend_http_settings_collection: Vec<
        AzureApplicationGatewaySubResource<AzureApplicationGatewayBackendHttpSettingsProperties>,
    >,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub backend_settings_collection: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub http_listeners:
        Vec<AzureApplicationGatewaySubResource<AzureApplicationGatewayHttpListenerProperties>>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub listeners: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub url_path_maps: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub request_routing_rules: Vec<
        AzureApplicationGatewaySubResource<AzureApplicationGatewayRequestRoutingRuleProperties>,
    >,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub routing_rules: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub probes: Vec<AzureApplicationGatewaySubResource<AzureApplicationGatewayProbeProperties>>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub rewrite_rule_sets: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub redirect_configurations: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub private_link_configurations: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub private_endpoint_connections: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub entra_jwt_validation_configs: Vec<serde_json::Value>,
    #[serde(default)]
    pub ssl_policy: Option<serde_json::Value>,
    #[serde(default)]
    pub web_application_firewall_configuration: Option<serde_json::Value>,
    #[serde(default)]
    pub enable_http2: Option<bool>,
    #[serde(default)]
    pub network_isolation_enabled: Option<bool>,
    #[serde(default)]
    pub default_predefined_ssl_policy: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub custom_error_configurations: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewaySku {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub capacity: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzureApplicationGatewaySubResource<T> {
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(rename = "type")]
    #[serde(default)]
    pub resource_type: Option<String>,
    pub properties: T,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayGatewayIpConfigurationProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub subnet: Option<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewaySslCertificateProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub key_vault_secret_id: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub http_listeners: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayTrustedRootCertificateProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub data: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub backend_http_settings: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayFrontendIpConfigurationProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub private_ip_allocation_method: Option<String>,
    #[serde(default)]
    pub public_ip_address: Option<AzureApplicationGatewayResourceReference>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub http_listeners: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayFrontendPortProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub http_listeners: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayBackendAddressPoolProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub backend_addresses: Vec<AzureApplicationGatewayBackendAddress>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub request_routing_rules: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayBackendAddress {
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub fqdn: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayBackendHttpSettingsProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub cookie_based_affinity: Option<String>,
    #[serde(default)]
    pub host_name: Option<String>,
    #[serde(default)]
    pub pick_host_name_from_backend_address: Option<bool>,
    #[serde(default)]
    pub dedicated_backend_connection: Option<bool>,
    #[serde(default)]
    pub validate_cert_chain_and_expiry: Option<bool>,
    #[serde(default)]
    pub validate_sni: Option<bool>,
    #[serde(default)]
    pub sni_name: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub request_timeout: Option<u32>,
    #[serde(default)]
    pub probe: Option<AzureApplicationGatewayResourceReference>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub trusted_root_certificates: Vec<AzureApplicationGatewayResourceReference>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub request_routing_rules: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayHttpListenerProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub frontend_ip_configuration: Option<AzureApplicationGatewayResourceReference>,
    #[serde(default)]
    pub frontend_port: Option<AzureApplicationGatewayResourceReference>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub ssl_certificate: Option<AzureApplicationGatewayResourceReference>,
    #[serde(default)]
    pub host_name: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub host_names: Vec<String>,
    #[serde(default)]
    pub require_server_name_indication: Option<bool>,
    #[serde(default)]
    pub enable_http3: Option<bool>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub custom_error_configurations: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub request_routing_rules: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayRequestRoutingRuleProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub rule_type: Option<String>,
    #[serde(default)]
    pub priority: Option<u32>,
    #[serde(default)]
    pub http_listener: Option<AzureApplicationGatewayResourceReference>,
    #[serde(default)]
    pub backend_address_pool: Option<AzureApplicationGatewayResourceReference>,
    #[serde(default)]
    pub backend_http_settings: Option<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayProbeProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub interval: Option<u32>,
    #[serde(default)]
    pub timeout: Option<u32>,
    #[serde(default)]
    pub unhealthy_threshold: Option<u32>,
    #[serde(default)]
    pub pick_host_name_from_backend_http_settings: Option<bool>,
    #[serde(default)]
    pub enable_probe_proxy_protocol_header: Option<bool>,
    #[serde(default)]
    pub min_servers: Option<u32>,
    #[serde(default)]
    pub match_value: Option<AzureApplicationGatewayProbeMatch>,
    #[serde(rename = "match")]
    #[serde(default)]
    pub probe_match: Option<AzureApplicationGatewayProbeMatch>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub backend_http_settings: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplicationGatewayProbeMatch {
    #[serde(default)]
    pub body: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub status_codes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzureApplicationGatewayResourceReference {
    pub id: String,
}
