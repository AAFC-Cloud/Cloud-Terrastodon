use crate::AzureDevOpsAccountLicenseType;
use crate::AzureDevOpsLicenseEntitlementUserReference;
use crate::AzureDevOpsLicenseRule;
use crate::AzureDevOpsLicenseRuleSource;
use crate::AzureDevOpsLicenseType;
use crate::AzureDevOpsUserId;
use crate::LastAccessedDate;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_types::ArbitraryJson;
use facet::Facet;

/// User entitlement returned by the [Azure DevOps 7.1 license search](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/search-user-entitlements?view=azure-devops-rest-7.1#userentitlement).
#[derive(Debug, Clone, Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsUserLicenseEntitlement {
    pub id: AzureDevOpsUserId,
    pub access_level: AzureDevOpsLicenseRule,
    /// The documented single-user response can omit the creation date.
    pub date_created: Option<DateTime<Utc>>,
    pub last_accessed_date: LastAccessedDate,
    pub user: AzureDevOpsLicenseEntitlementUserReference,
    // License-only searches do not request these sections. Preserve them as
    // opaque data when present, including the empty arrays Azure can return.
    pub extensions: Option<Vec<ArbitraryJson>>,
    pub group_assignments: Option<Vec<ArbitraryJson>>,
    pub project_entitlements: Option<Vec<ArbitraryJson>>,
}

impl AzureDevOpsUserLicenseEntitlement {
    /// Resolve the effective license using only the family's active source.
    pub fn license(&self) -> AzureDevOpsLicenseType {
        match &self.access_level.licensing_source {
            AzureDevOpsLicenseRuleSource::None => {
                AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::None)
            }
            AzureDevOpsLicenseRuleSource::Account => {
                AzureDevOpsLicenseType::Account(self.access_level.account_license_type.clone())
            }
            AzureDevOpsLicenseRuleSource::Msdn => {
                AzureDevOpsLicenseType::Msdn(self.access_level.msdn_license_type.clone())
            }
            AzureDevOpsLicenseRuleSource::Other(source) => AzureDevOpsLicenseType::Other(format!(
                "{source}: {}",
                self.access_level.license_display_name
            )),
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsUserLicenseEntitlement);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsUserLicenseEntitlement);
