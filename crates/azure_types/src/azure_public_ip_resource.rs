use crate::AzureLocationName;
use crate::AzurePublicIpResourceId;
use crate::AzurePublicIpResourceName;
use crate::AzureTenantId;
use facet_json::RawJson;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePublicIpResource {
    pub id: AzurePublicIpResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzurePublicIpResourceName,
    #[facet(default)]
    pub sku: Option<AzurePublicIpSku>,
    pub location: AzureLocationName,
    #[facet(default)]
    pub tags: HashMap<String, String>,
    pub properties: AzurePublicIpResourceProperties,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
pub struct AzurePublicIpSku {
    pub name: String,
    pub tier: String,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePublicIpResourceProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub resource_guid: Option<String>,
    #[facet(default, opaque, proxy = crate::OptionalIpAddrProxy)]
    pub ip_address: Option<IpAddr>,
    #[facet(default)]
    pub public_ip_address_version: Option<String>,
    #[facet(default)]
    pub public_ip_allocation_method: Option<String>,
    #[facet(default)]
    pub idle_timeout_in_minutes: Option<u16>,
    #[facet(default)]
    pub dns_settings: Option<AzurePublicIpDnsSettings>,
    #[facet(default)]
    pub ip_tags: Vec<RawJson<'static>>,
    #[facet(default)]
    pub ip_configuration: Option<AzurePublicIpConfigurationReference>,
    #[facet(default)]
    pub ddos_settings: Option<AzurePublicIpDdosSettings>,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePublicIpDnsSettings {
    #[facet(default)]
    pub domain_name_label: Option<String>,
    #[facet(default)]
    pub fqdn: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
pub struct AzurePublicIpConfigurationReference {
    pub id: String,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePublicIpDdosSettings {
    #[facet(default)]
    pub protection_mode: Option<String>,
}
