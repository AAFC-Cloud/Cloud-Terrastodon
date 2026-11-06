use crate::AzureResourceSystemData;
use crate::PolicyRemediationId;
use crate::PolicyRemediationName;
use crate::PolicyRemediationProperties;
use crate::ResourceType;
use arbitrary::Arbitrary;
use facet::Facet;

/// A remediation task returned by Azure Policy Insights.
///
/// [2024-10-01 schema](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01#remediation).
/// A successful creation response acknowledges the task; it does not establish
/// that its deployments completed. Task identity and properties are required.
#[derive(Debug, Clone, PartialEq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct PolicyRemediation {
    pub id: PolicyRemediationId,
    pub name: PolicyRemediationName,
    #[facet(rename = "type")]
    pub resource_type: ResourceType,
    /// Omitted by the [management-group create-or-update 200 response example](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-management-group?view=rest-policyinsights-2024-10-01#examples).
    pub system_data: Option<AzureResourceSystemData>,
    pub properties: PolicyRemediationProperties,
}

cloud_terrastodon_registry::register_thing!(PolicyRemediation);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediation);
