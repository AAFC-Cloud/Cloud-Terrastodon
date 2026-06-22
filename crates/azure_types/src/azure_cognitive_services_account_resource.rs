use crate::AzureCognitiveServicesAccountResourceId;
use crate::AzureCognitiveServicesAccountResourceName;
use crate::AzureLocationName;
use crate::AzureTenantId;
use facet_json::RawJson;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureCognitiveServicesAccountResource {
    pub id: AzureCognitiveServicesAccountResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzureCognitiveServicesAccountResourceName,
    #[facet(default)]
    pub kind: Option<String>,
    #[facet(default)]
    pub sku: Option<AzureCognitiveServicesSku>,
    pub location: AzureLocationName,
    #[facet(default)]
    pub tags: HashMap<String, String>,
    pub properties: AzureCognitiveServicesAccountResourceProperties,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureCognitiveServicesAccountResourceProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub public_network_access: Option<String>,
    #[facet(default)]
    pub private_endpoint_connections: Vec<RawJson<'static>>,
    #[facet(default)]
    pub network_acls: Option<AzureCognitiveServicesNetworkAcls>,
    #[facet(default)]
    pub internal_id: Option<String>,
    #[facet(default)]
    pub capabilities: Vec<AzureCognitiveServicesCapability>,
    #[facet(default)]
    pub endpoint: Option<String>,
    #[facet(default)]
    pub endpoints: HashMap<String, String>,
    #[facet(default)]
    pub custom_sub_domain_name: Option<String>,
    #[facet(default)]
    pub date_created: Option<String>,
    #[facet(default)]
    pub call_rate_limit: Option<AzureCognitiveServicesCallRateLimit>,
    #[facet(default)]
    pub allow_project_management: Option<bool>,
    #[facet(default)]
    pub is_migrated: Option<bool>,
    #[facet(default)]
    pub api_properties: Option<RawJson<'static>>,
    #[facet(default)]
    pub associated_projects: Vec<String>,
    #[facet(default)]
    pub default_project: Option<String>,
    #[facet(default)]
    pub stored_completions_disabled: Option<bool>,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureCognitiveServicesSku {
    pub name: String,
    #[facet(default)]
    pub tier: Option<String>,
    #[facet(default)]
    pub size: Option<String>,
    #[facet(default)]
    pub family: Option<String>,
    #[facet(default)]
    pub capacity: Option<i32>,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureCognitiveServicesNetworkAcls {
    #[facet(default)]
    pub virtual_network_rules: Vec<RawJson<'static>>,
    #[facet(default)]
    pub default_action: Option<String>,
    #[facet(default)]
    pub ip_rules: Vec<RawJson<'static>>,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
pub struct AzureCognitiveServicesCapability {
    pub name: String,
    #[facet(default)]
    pub value: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
pub struct AzureCognitiveServicesCallRateLimit {
    #[facet(default)]
    pub rules: Vec<AzureCognitiveServicesThrottlingRule>,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureCognitiveServicesThrottlingRule {
    #[facet(default)]
    pub match_patterns: Vec<AzureCognitiveServicesRequestMatchPattern>,
    #[facet(default)]
    pub renewal_period: Option<u64>,
    #[facet(default)]
    pub count: Option<u64>,
    #[facet(default)]
    pub min_count: Option<u64>,
    #[facet(default)]
    pub key: Option<String>,
    #[facet(default)]
    pub dynamic_throttling_enabled: Option<bool>,
}

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
pub struct AzureCognitiveServicesRequestMatchPattern {
    #[facet(default)]
    pub method: Option<String>,
    #[facet(default)]
    pub path: Option<String>,
}
