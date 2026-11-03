use crate::AzureLocationName;
use crate::ResourceId;
use crate::Scope;
use crate::ScopeImpl;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;

/// Location and resource selections reported by a remediation task.
///
/// [2024-10-01 schema](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01#remediationfilters).
/// `resourceIds`, when supplied, contains 1–100 IDs and is incompatible with
/// `ReEvaluateCompliance`. This response record preserves omitted filters.
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct PolicyRemediationFilters {
    /// Unspecified locations allow resources from any region, as described by
    /// the [filter contract](https://learn.microsoft.com/en-us/azure/governance/policy/concepts/remediation-structure#remediation-filters).
    pub locations: Option<Vec<AzureLocationName>>,
    /// An explicit resource selection is optional and cannot be used with
    /// `ReEvaluateCompliance`; location-only filters omit it.
    pub resource_ids: Option<Vec<ScopeImpl>>,
}

impl<'a> Arbitrary<'a> for PolicyRemediationFilters {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            locations: u.arbitrary()?,
            resource_ids: if u.arbitrary()? {
                Some(vec![ResourceId::arbitrary(u)?.as_scope_impl()])
            } else {
                None
            },
        })
    }
}

cloud_terrastodon_registry::register_thing!(PolicyRemediationFilters);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationFilters);
