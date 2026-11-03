use crate::PolicyAssignmentId;
use crate::PolicyDefinitionReferenceId;
use crate::PolicyRemediationDeploymentSummary;
use crate::PolicyRemediationFailureThreshold;
use crate::PolicyRemediationFilters;
use crate::PolicyRemediationProvisioningState;
use crate::PolicyRemediationResourceDiscoveryMode;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use chrono::DateTime;
use chrono::Utc;

/// Configuration and progress of a submitted remediation task.
///
/// [2024-10-01 schema](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01#remediation).
/// The task's identity, state, timestamps, and deployment summary are required.
/// The [basic creation responses](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01#examples)
/// omit filters and limit overrides; their absence does not establish an echoed
/// default value.
#[derive(Debug, Clone, PartialEq, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct PolicyRemediationProperties {
    pub policy_assignment_id: PolicyAssignmentId,
    /// Initiative assignments select a definition; single-definition assignments
    /// omit this field, as shown by the basic creation responses.
    pub policy_definition_reference_id: Option<PolicyDefinitionReferenceId>,
    pub resource_discovery_mode: PolicyRemediationResourceDiscoveryMode,
    pub provisioning_state: PolicyRemediationProvisioningState,
    pub created_on: DateTime<Utc>,
    pub last_updated_on: DateTime<Utc>,
    /// Absent from basic creation responses when no narrowing filters are supplied.
    pub filters: Option<PolicyRemediationFilters>,
    pub deployment_status: PolicyRemediationDeploymentSummary,
    pub status_message: String,
    /// Opaque activity-log correlation identity; the schema has no UUID constraint.
    pub correlation_id: String,
    /// Basic creation responses omit this limit even though the service applies a default.
    pub resource_count: Option<u32>,
    /// Basic creation responses omit this concurrency limit when it is not supplied.
    pub parallel_deployments: Option<u32>,
    /// An optional override omitted from the basic creation responses.
    pub failure_threshold: Option<PolicyRemediationFailureThreshold>,
}

impl<'a> Arbitrary<'a> for PolicyRemediationProperties {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let resource_discovery_mode = u.arbitrary()?;
        let mut filters: Option<PolicyRemediationFilters> = u.arbitrary()?;
        if matches!(
            resource_discovery_mode,
            PolicyRemediationResourceDiscoveryMode::ReEvaluateCompliance
        ) && let Some(filters) = &mut filters
        {
            filters.resource_ids = None;
        }
        Ok(Self {
            policy_assignment_id: u.arbitrary()?,
            policy_definition_reference_id: u.arbitrary()?,
            resource_discovery_mode,
            provisioning_state: u.arbitrary()?,
            created_on: u.arbitrary()?,
            last_updated_on: u.arbitrary()?,
            filters,
            deployment_status: u.arbitrary()?,
            status_message: u.arbitrary()?,
            correlation_id: u.arbitrary()?,
            resource_count: Some(u.int_in_range(1..=50_000)?),
            parallel_deployments: Some(u.int_in_range(1..=30)?),
            failure_threshold: u.arbitrary()?,
        })
    }
}

cloud_terrastodon_registry::register_thing!(PolicyRemediationProperties);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationProperties);
