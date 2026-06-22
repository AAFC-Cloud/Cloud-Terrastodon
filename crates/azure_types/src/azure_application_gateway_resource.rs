use crate::AzureApplicationGatewayResourceId;
use crate::AzureApplicationGatewayResourceName;
use crate::AzureLocationName;
use crate::AzureTenantId;
use facet_json::RawJson;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResource {
    pub id: AzureApplicationGatewayResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzureApplicationGatewayResourceName,
    pub location: AzureLocationName,
    #[facet(default, opaque, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
    #[facet(default)]
    pub identity: Option<AzureApplicationGatewayIdentity>,
    pub properties: AzureApplicationGatewayResourceProperties,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayIdentity {
    #[facet(rename = "type")]
    pub identity_type: String,
    #[facet(
        default,
        opaque,
        proxy = crate::HashMapDefaultNullProxy<AzureApplicationGatewayUserAssignedIdentity>
    )]
    pub user_assigned_identities: HashMap<String, AzureApplicationGatewayUserAssignedIdentity>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayUserAssignedIdentity {
    #[facet(default)]
    pub principal_id: Option<String>,
    #[facet(default)]
    pub client_id: Option<String>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayResourceProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub resource_guid: Option<String>,
    #[facet(default)]
    pub sku: Option<AzureApplicationGatewaySku>,
    #[facet(default)]
    pub operational_state: Option<String>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<
            AzureApplicationGatewaySubResource<
                AzureApplicationGatewayGatewayIpConfigurationProperties,
            >,
        >
    )]
    pub gateway_ip_configurations: Vec<
        AzureApplicationGatewaySubResource<AzureApplicationGatewayGatewayIpConfigurationProperties>,
    >,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<
            AzureApplicationGatewaySubResource<AzureApplicationGatewaySslCertificateProperties>,
        >
    )]
    pub ssl_certificates:
        Vec<AzureApplicationGatewaySubResource<AzureApplicationGatewaySslCertificateProperties>>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<
            AzureApplicationGatewaySubResource<
                AzureApplicationGatewayTrustedRootCertificateProperties,
            >,
        >
    )]
    pub trusted_root_certificates: Vec<
        AzureApplicationGatewaySubResource<AzureApplicationGatewayTrustedRootCertificateProperties>,
    >,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub trusted_client_certificates: Vec<RawJson<'static>>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub ssl_profiles: Vec<RawJson<'static>>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<
            AzureApplicationGatewaySubResource<
                AzureApplicationGatewayFrontendIpConfigurationProperties,
            >,
        >
    )]
    pub frontend_ip_configurations: Vec<
        AzureApplicationGatewaySubResource<
            AzureApplicationGatewayFrontendIpConfigurationProperties,
        >,
    >,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<
            AzureApplicationGatewaySubResource<AzureApplicationGatewayFrontendPortProperties>,
        >
    )]
    pub frontend_ports:
        Vec<AzureApplicationGatewaySubResource<AzureApplicationGatewayFrontendPortProperties>>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<
            AzureApplicationGatewaySubResource<AzureApplicationGatewayBackendAddressPoolProperties>,
        >
    )]
    pub backend_address_pools: Vec<
        AzureApplicationGatewaySubResource<AzureApplicationGatewayBackendAddressPoolProperties>,
    >,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub load_distribution_policies: Vec<RawJson<'static>>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<
            AzureApplicationGatewaySubResource<AzureApplicationGatewayBackendHttpSettingsProperties>,
        >
    )]
    pub backend_http_settings_collection: Vec<
        AzureApplicationGatewaySubResource<AzureApplicationGatewayBackendHttpSettingsProperties>,
    >,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub backend_settings_collection: Vec<RawJson<'static>>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<
            AzureApplicationGatewaySubResource<AzureApplicationGatewayHttpListenerProperties>,
        >
    )]
    pub http_listeners:
        Vec<AzureApplicationGatewaySubResource<AzureApplicationGatewayHttpListenerProperties>>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub listeners: Vec<RawJson<'static>>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub url_path_maps: Vec<RawJson<'static>>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<
            AzureApplicationGatewaySubResource<
                AzureApplicationGatewayRequestRoutingRuleProperties,
            >,
        >
    )]
    pub request_routing_rules: Vec<
        AzureApplicationGatewaySubResource<AzureApplicationGatewayRequestRoutingRuleProperties>,
    >,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub routing_rules: Vec<RawJson<'static>>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<
            AzureApplicationGatewaySubResource<AzureApplicationGatewayProbeProperties>,
        >
    )]
    pub probes: Vec<AzureApplicationGatewaySubResource<AzureApplicationGatewayProbeProperties>>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub rewrite_rule_sets: Vec<RawJson<'static>>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub redirect_configurations: Vec<RawJson<'static>>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub private_link_configurations: Vec<RawJson<'static>>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub private_endpoint_connections: Vec<RawJson<'static>>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub entra_jwt_validation_configs: Vec<RawJson<'static>>,
    #[facet(default)]
    pub ssl_policy: Option<RawJson<'static>>,
    #[facet(default)]
    pub web_application_firewall_configuration: Option<RawJson<'static>>,
    #[facet(default)]
    pub enable_http2: Option<bool>,
    #[facet(default)]
    pub network_isolation_enabled: Option<bool>,
    #[facet(default)]
    pub default_predefined_ssl_policy: Option<String>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub custom_error_configurations: Vec<RawJson<'static>>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewaySku {
    #[facet(default)]
    pub name: Option<String>,
    #[facet(default)]
    pub tier: Option<String>,
    #[facet(default)]
    pub family: Option<String>,
    #[facet(default)]
    pub capacity: Option<u32>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct AzureApplicationGatewaySubResource<T> {
    pub name: String,
    pub id: String,
    #[facet(default)]
    pub etag: Option<String>,
    #[facet(rename = "type", default)]
    pub resource_type: Option<String>,
    pub properties: T,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayGatewayIpConfigurationProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub subnet: Option<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewaySslCertificateProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub key_vault_secret_id: Option<String>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureApplicationGatewayResourceReference>
    )]
    pub http_listeners: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayTrustedRootCertificateProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub data: Option<String>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureApplicationGatewayResourceReference>
    )]
    pub backend_http_settings: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayFrontendIpConfigurationProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub private_ip_allocation_method: Option<String>,
    #[facet(default)]
    pub public_ip_address: Option<AzureApplicationGatewayResourceReference>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureApplicationGatewayResourceReference>
    )]
    pub http_listeners: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayFrontendPortProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub port: Option<u16>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureApplicationGatewayResourceReference>
    )]
    pub http_listeners: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayBackendAddressPoolProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureApplicationGatewayBackendAddress>
    )]
    pub backend_addresses: Vec<AzureApplicationGatewayBackendAddress>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureApplicationGatewayResourceReference>
    )]
    pub request_routing_rules: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayBackendAddress {
    #[facet(default)]
    pub ip_address: Option<String>,
    #[facet(default)]
    pub fqdn: Option<String>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayBackendHttpSettingsProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub port: Option<u16>,
    #[facet(default)]
    pub protocol: Option<String>,
    #[facet(default)]
    pub cookie_based_affinity: Option<String>,
    #[facet(default)]
    pub host_name: Option<String>,
    #[facet(default)]
    pub pick_host_name_from_backend_address: Option<bool>,
    #[facet(default)]
    pub dedicated_backend_connection: Option<bool>,
    #[facet(default)]
    pub validate_cert_chain_and_expiry: Option<bool>,
    #[facet(default)]
    pub validate_sni: Option<bool>,
    #[facet(default)]
    pub sni_name: Option<String>,
    #[facet(default)]
    pub path: Option<String>,
    #[facet(default)]
    pub request_timeout: Option<u32>,
    #[facet(default)]
    pub probe: Option<AzureApplicationGatewayResourceReference>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureApplicationGatewayResourceReference>
    )]
    pub trusted_root_certificates: Vec<AzureApplicationGatewayResourceReference>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureApplicationGatewayResourceReference>
    )]
    pub request_routing_rules: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayHttpListenerProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub frontend_ip_configuration: Option<AzureApplicationGatewayResourceReference>,
    #[facet(default)]
    pub frontend_port: Option<AzureApplicationGatewayResourceReference>,
    #[facet(default)]
    pub protocol: Option<String>,
    #[facet(default)]
    pub ssl_certificate: Option<AzureApplicationGatewayResourceReference>,
    #[facet(default)]
    pub host_name: Option<String>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<String>)]
    pub host_names: Vec<String>,
    #[facet(default)]
    pub require_server_name_indication: Option<bool>,
    #[facet(default)]
    pub enable_http3: Option<bool>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RawJson<'static>>)]
    pub custom_error_configurations: Vec<RawJson<'static>>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureApplicationGatewayResourceReference>
    )]
    pub request_routing_rules: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayRequestRoutingRuleProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub rule_type: Option<String>,
    #[facet(default)]
    pub priority: Option<u32>,
    #[facet(default)]
    pub http_listener: Option<AzureApplicationGatewayResourceReference>,
    #[facet(default)]
    pub backend_address_pool: Option<AzureApplicationGatewayResourceReference>,
    #[facet(default)]
    pub backend_http_settings: Option<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayProbeProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub protocol: Option<String>,
    #[facet(default)]
    pub host: Option<String>,
    #[facet(default)]
    pub path: Option<String>,
    #[facet(default)]
    pub interval: Option<u32>,
    #[facet(default)]
    pub timeout: Option<u32>,
    #[facet(default)]
    pub unhealthy_threshold: Option<u32>,
    #[facet(default)]
    pub pick_host_name_from_backend_http_settings: Option<bool>,
    #[facet(default)]
    pub enable_probe_proxy_protocol_header: Option<bool>,
    #[facet(default)]
    pub min_servers: Option<u32>,
    #[facet(default)]
    pub match_value: Option<AzureApplicationGatewayProbeMatch>,
    #[facet(rename = "match", default)]
    pub probe_match: Option<AzureApplicationGatewayProbeMatch>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureApplicationGatewayResourceReference>
    )]
    pub backend_http_settings: Vec<AzureApplicationGatewayResourceReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureApplicationGatewayProbeMatch {
    #[facet(default)]
    pub body: Option<String>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<String>)]
    pub status_codes: Vec<String>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct AzureApplicationGatewayResourceReference {
    pub id: String,
}
