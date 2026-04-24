use crate::AzureCognitiveServicesAccountResourceId;
use crate::AzureCognitiveServicesAccountResourceName;
use crate::AzureLocationName;
use crate::AzureTenantId;
use crate::serde_helpers::deserialize_default_if_null;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureCognitiveServicesAccountResource {
    pub id: AzureCognitiveServicesAccountResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzureCognitiveServicesAccountResourceName,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub sku: Option<AzureCognitiveServicesSku>,
    pub location: AzureLocationName,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
    pub properties: AzureCognitiveServicesAccountResourceProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureCognitiveServicesAccountResourceProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub public_network_access: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub private_endpoint_connections: Vec<serde_json::Value>,
    #[serde(default)]
    pub network_acls: Option<AzureCognitiveServicesNetworkAcls>,
    #[serde(default)]
    pub internal_id: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub capabilities: Vec<AzureCognitiveServicesCapability>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub endpoints: HashMap<String, String>,
    #[serde(default)]
    pub custom_sub_domain_name: Option<String>,
    #[serde(default)]
    pub date_created: Option<String>,
    #[serde(default)]
    pub call_rate_limit: Option<AzureCognitiveServicesCallRateLimit>,
    #[serde(default)]
    pub allow_project_management: Option<bool>,
    #[serde(default)]
    pub is_migrated: Option<bool>,
    #[serde(default)]
    pub api_properties: Option<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub associated_projects: Vec<String>,
    #[serde(default)]
    pub default_project: Option<String>,
    #[serde(default)]
    pub stored_completions_disabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureCognitiveServicesSku {
    pub name: String,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub capacity: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureCognitiveServicesNetworkAcls {
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub virtual_network_rules: Vec<serde_json::Value>,
    #[serde(default)]
    pub default_action: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub ip_rules: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzureCognitiveServicesCapability {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzureCognitiveServicesCallRateLimit {
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub rules: Vec<AzureCognitiveServicesThrottlingRule>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureCognitiveServicesThrottlingRule {
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub match_patterns: Vec<AzureCognitiveServicesRequestMatchPattern>,
    #[serde(default)]
    pub renewal_period: Option<u64>,
    #[serde(default)]
    pub count: Option<u64>,
    #[serde(default)]
    pub min_count: Option<u64>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub dynamic_throttling_enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzureCognitiveServicesRequestMatchPattern {
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}
