use arbitrary::Arbitrary;

/// How a remediation identifies noncompliant resources.
///
/// The [2024-10-01 schema](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01#resourcediscoverymode)
/// defaults to `ExistingNonCompliant` when the request omits this property.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum PolicyRemediationResourceDiscoveryMode {
    ExistingNonCompliant,
    ReEvaluateCompliance,
    #[arbitrary(skip)]
    Other(String),
}

impl From<String> for PolicyRemediationResourceDiscoveryMode {
    fn from(value: String) -> Self {
        match value.as_str() {
            "ExistingNonCompliant" => Self::ExistingNonCompliant,
            "ReEvaluateCompliance" => Self::ReEvaluateCompliance,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Other(value)
            }
        }
    }
}

impl From<&PolicyRemediationResourceDiscoveryMode> for String {
    fn from(value: &PolicyRemediationResourceDiscoveryMode) -> Self {
        match value {
            PolicyRemediationResourceDiscoveryMode::ExistingNonCompliant => "ExistingNonCompliant",
            PolicyRemediationResourceDiscoveryMode::ReEvaluateCompliance => "ReEvaluateCompliance",
            PolicyRemediationResourceDiscoveryMode::Other(value) => value.as_str(),
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(PolicyRemediationResourceDiscoveryMode);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationResourceDiscoveryMode);
