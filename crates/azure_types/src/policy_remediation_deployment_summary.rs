use arbitrary::Arbitrary;
use arbitrary::Unstructured;

/// Counts of deployments created by a remediation task.
///
/// [2024-10-01 schema](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01#remediationdeploymentsummary).
/// All three counters are returned together, including zero counts.
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct PolicyRemediationDeploymentSummary {
    pub total_deployments: u32,
    pub successful_deployments: u32,
    pub failed_deployments: u32,
}

impl<'a> Arbitrary<'a> for PolicyRemediationDeploymentSummary {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let total = u.int_in_range(0..=i32::MAX as u32)?;
        let successful = u.int_in_range(0..=total)?;
        let failed = u.int_in_range(0..=total - successful)?;
        Ok(Self {
            total_deployments: total,
            successful_deployments: successful,
            failed_deployments: failed,
        })
    }
}

cloud_terrastodon_registry::register_thing!(PolicyRemediationDeploymentSummary);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationDeploymentSummary);
