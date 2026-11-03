use crate::PolicyAssignmentId;
use crate::PolicyDefinitionReferenceId;
use arbitrary::Arbitrary;

/// Assignment selection sent when creating a remediation with service defaults.
///
/// [2024-10-01 request schema](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01#request-body).
/// An initiative assignment requires its local definition reference ID.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct PolicyRemediationCreateProperties {
    pub policy_assignment_id: PolicyAssignmentId,
    #[facet(skip_serializing_if = Option::is_none)]
    pub policy_definition_reference_id: Option<PolicyDefinitionReferenceId>,
}

cloud_terrastodon_registry::register_thing!(PolicyRemediationCreateProperties);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationCreateProperties);
