use crate::prelude::AzureDevOpsDescriptor;
use crate::prelude::AzureDevOpsLicenseRule;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsGroupLicenseEntitlement {
    pub extension_rules: Vec<Value>,
    pub group: AzureDevOpsGroupLicenseEntitlementGroupReference,
    pub id: Uuid,
    pub last_executed: DateTime<Utc>,
    pub license_rule: AzureDevOpsLicenseRule,
    pub members: Option<Value>,
    pub project_entitlements: Vec<Value>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsGroupLicenseEntitlementGroupReference {
    #[serde(rename = "_links")]
    pub links: Value,
    pub description: String,
    pub descriptor: AzureDevOpsDescriptor,
    pub display_name: String,
    pub domain: String,
    pub mail_address: Option<String>,
    pub origin: String,
    pub origin_id: String,
    pub principal_name: String,
    pub subject_kind: String,
    pub url: String,
}
