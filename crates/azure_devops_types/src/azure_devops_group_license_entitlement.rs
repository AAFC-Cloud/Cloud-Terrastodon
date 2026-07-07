use crate::AzureDevOpsDescriptor;
use crate::AzureDevOpsLicenseRule;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_types::ArbitraryJson;
use uuid::Uuid;

#[derive(Debug, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsGroupLicenseEntitlement {
    pub extension_rules: Vec<ArbitraryJson>,
    pub group: AzureDevOpsGroupLicenseEntitlementGroupReference,
    pub id: Uuid,
    pub last_executed: DateTime<Utc>,
    pub license_rule: AzureDevOpsLicenseRule,
    pub members: Option<ArbitraryJson>,
    pub project_entitlements: Vec<ArbitraryJson>,
    pub status: String,
}

#[derive(Debug, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsGroupLicenseEntitlementGroupReference {
    #[facet(rename = "_links")]
    pub links: ArbitraryJson,
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

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupLicenseEntitlement);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupLicenseEntitlement);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsGroupLicenseEntitlement>);
