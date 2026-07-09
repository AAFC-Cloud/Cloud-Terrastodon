use crate::AzureCognitiveServicesAccountDeploymentId;
use crate::AzureCognitiveServicesAccountDeploymentName;
use arbitrary::Arbitrary;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureCognitiveServicesAccountDeployment {
    pub id: AzureCognitiveServicesAccountDeploymentId,
    pub name: AzureCognitiveServicesAccountDeploymentName,
    #[facet(rename = "type")]
    pub resource_type: String,
    #[facet(default)]
    pub sku: Option<AzureCognitiveServicesAccountDeploymentSku>,
    pub properties: AzureCognitiveServicesAccountDeploymentProperties,
    #[facet(default)]
    pub system_data: Option<AzureCognitiveServicesSystemData>,
    #[facet(default)]
    pub etag: Option<String>,
    #[facet(default, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureCognitiveServicesAccountDeploymentSku {
    pub name: String,
    #[facet(default)]
    pub capacity: Option<i32>,
    #[facet(default)]
    pub tier: Option<String>,
    #[facet(default)]
    pub family: Option<String>,
    #[facet(default)]
    pub size: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureCognitiveServicesAccountDeploymentProperties {
    #[facet(default)]
    pub model: Option<AzureCognitiveServicesAccountDeploymentModel>,
    #[facet(default)]
    pub version_upgrade_option: Option<String>,
    #[facet(default)]
    pub current_capacity: Option<i32>,
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub rai_policy_name: Option<String>,
    #[facet(default)]
    pub parent_deployment_name: Option<String>,
    #[facet(default)]
    pub dynamic_throttling_enabled: Option<bool>,
    #[facet(default)]
    pub service_tier: Option<String>,
    #[facet(default)]
    pub spillover_deployment_name: Option<String>,
    #[facet(default)]
    pub capabilities: Option<HashMap<String, String>>,
    #[facet(
        default,
                proxy = crate::VecDefaultNullProxy<AzureCognitiveServicesAccountDeploymentRateLimit>
    )]
    pub rate_limits: Vec<AzureCognitiveServicesAccountDeploymentRateLimit>,
}

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
pub struct AzureCognitiveServicesAccountDeploymentModel {
    #[facet(default)]
    pub format: Option<String>,
    #[facet(default)]
    pub name: Option<String>,
    #[facet(default)]
    pub version: Option<String>,
    #[facet(default)]
    pub publisher: Option<String>,
    #[facet(default)]
    pub source: Option<String>,
    #[facet(default)]
    pub source_account: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureCognitiveServicesAccountDeploymentRateLimit {
    #[facet(default)]
    pub key: Option<String>,
    #[facet(default)]
    pub renewal_period: Option<u64>,
    #[facet(default)]
    pub count: Option<u64>,
}

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureCognitiveServicesSystemData {
    #[facet(default)]
    pub created_by: Option<String>,
    #[facet(default)]
    pub created_by_type: Option<String>,
    #[facet(default)]
    pub created_at: Option<String>,
    #[facet(default)]
    pub last_modified_by: Option<String>,
    #[facet(default)]
    pub last_modified_by_type: Option<String>,
    #[facet(default)]
    pub last_modified_at: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
pub struct AzureCognitiveServicesAccountDeploymentListResult {
    #[facet(
        default,

        proxy = crate::VecDefaultNullProxy<AzureCognitiveServicesAccountDeployment>
    )]
    pub value: Vec<AzureCognitiveServicesAccountDeployment>,
}

cloud_terrastodon_registry::register_thing!(AzureCognitiveServicesAccountDeployment);
cloud_terrastodon_registry::register_arbitrary!(AzureCognitiveServicesAccountDeployment);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureCognitiveServicesAccountDeployment>);
