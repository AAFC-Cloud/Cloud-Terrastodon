use crate::PolicyRemediationCreateProperties;
use arbitrary::Arbitrary;

/// The nested ARM body used to submit a remediation task.
///
/// [2024-10-01 request schema](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01#request-body).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
pub struct PolicyRemediationCreateBody {
    pub properties: PolicyRemediationCreateProperties,
}

cloud_terrastodon_registry::register_thing!(PolicyRemediationCreateBody);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationCreateBody);
