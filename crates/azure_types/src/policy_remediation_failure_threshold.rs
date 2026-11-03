use crate::PolicyRemediationFailurePercentage;
use arbitrary::Arbitrary;

/// Failure policy reported by a remediation task.
///
/// [2024-10-01 schema](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01#remediationpropertiesfailurethreshold).
/// When the threshold object is present, its percentage supplies the threshold.
#[derive(Debug, Clone, PartialEq, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct PolicyRemediationFailureThreshold {
    pub percentage: PolicyRemediationFailurePercentage,
}

cloud_terrastodon_registry::register_thing!(PolicyRemediationFailureThreshold);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationFailureThreshold);
