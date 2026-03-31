use crate::AzureLocationName;
use crate::AzurePublicIpResourceId;
use crate::AzurePublicIpResourceName;
use crate::AzureTenantId;
use crate::serde_helpers::deserialize_default_if_null;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePublicIpResource {
    pub id: AzurePublicIpResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzurePublicIpResourceName,
    #[serde(default)]
    pub sku: Option<AzurePublicIpSku>,
    pub location: AzureLocationName,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
    pub properties: AzurePublicIpResourceProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzurePublicIpSku {
    pub name: String,
    pub tier: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePublicIpResourceProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub resource_guid: Option<String>,
    #[serde(default)]
    pub ip_address: Option<IpAddr>,
    #[serde(default)]
    pub public_ip_address_version: Option<String>,
    #[serde(default)]
    pub public_ip_allocation_method: Option<String>,
    #[serde(default)]
    pub idle_timeout_in_minutes: Option<u16>,
    #[serde(default)]
    pub dns_settings: Option<AzurePublicIpDnsSettings>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub ip_tags: Vec<serde_json::Value>,
    #[serde(default)]
    pub ip_configuration: Option<AzurePublicIpConfigurationReference>,
    #[serde(default)]
    pub ddos_settings: Option<AzurePublicIpDdosSettings>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePublicIpDnsSettings {
    #[serde(default)]
    pub domain_name_label: Option<String>,
    #[serde(default)]
    pub fqdn: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzurePublicIpConfigurationReference {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePublicIpDdosSettings {
    #[serde(default)]
    pub protection_mode: Option<String>,
}
