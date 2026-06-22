use crate::AzureDevOpsDescriptor;
use crate::AzureDevOpsLicenseRule;
use chrono::DateTime;
use chrono::Utc;
use facet_json::RawJson;
use uuid::Uuid;

#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsGroupLicenseEntitlement {
    pub extension_rules: Vec<RawJson<'static>>,
    pub group: AzureDevOpsGroupLicenseEntitlementGroupReference,
    pub id: Uuid,
    pub last_executed: DateTime<Utc>,
    pub license_rule: AzureDevOpsLicenseRule,
    pub members: Option<RawJson<'static>>,
    pub project_entitlements: Vec<RawJson<'static>>,
    pub status: String,
}

#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsGroupLicenseEntitlementGroupReference {
    #[facet(rename = "_links")]
    pub links: RawJson<'static>,
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
