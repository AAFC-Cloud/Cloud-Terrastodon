use crate::AzureCognitiveServicesAccountDeploymentId;
use crate::AzureCognitiveServicesAccountDeploymentName;
use crate::serde_helpers::deserialize_default_if_null;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureCognitiveServicesAccountDeployment {
    pub id: AzureCognitiveServicesAccountDeploymentId,
    pub name: AzureCognitiveServicesAccountDeploymentName,
    #[serde(rename = "type")]
    pub resource_type: String,
    #[serde(default)]
    pub sku: Option<AzureCognitiveServicesAccountDeploymentSku>,
    pub properties: AzureCognitiveServicesAccountDeploymentProperties,
    #[serde(default)]
    pub system_data: Option<AzureCognitiveServicesSystemData>,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureCognitiveServicesAccountDeploymentSku {
    pub name: String,
    #[serde(default)]
    pub capacity: Option<i32>,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub size: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureCognitiveServicesAccountDeploymentProperties {
    #[serde(default)]
    pub model: Option<AzureCognitiveServicesAccountDeploymentModel>,
    #[serde(default)]
    pub version_upgrade_option: Option<String>,
    #[serde(default)]
    pub current_capacity: Option<i32>,
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub rai_policy_name: Option<String>,
    #[serde(default)]
    pub parent_deployment_name: Option<String>,
    #[serde(default)]
    pub dynamic_throttling_enabled: Option<bool>,
    #[serde(default)]
    pub service_tier: Option<String>,
    #[serde(default)]
    pub spillover_deployment_name: Option<String>,
    #[serde(default)]
    pub capabilities: Option<HashMap<String, String>>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub rate_limits: Vec<AzureCognitiveServicesAccountDeploymentRateLimit>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzureCognitiveServicesAccountDeploymentModel {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub source_account: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureCognitiveServicesAccountDeploymentRateLimit {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub renewal_period: Option<u64>,
    #[serde(default)]
    pub count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureCognitiveServicesSystemData {
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub created_by_type: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_modified_by: Option<String>,
    #[serde(default)]
    pub last_modified_by_type: Option<String>,
    #[serde(default)]
    pub last_modified_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzureCognitiveServicesAccountDeploymentListResult {
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub value: Vec<AzureCognitiveServicesAccountDeployment>,
}
