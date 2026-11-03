use arbitrary::Arbitrary;

/// Task-wide status, distinct from any individual deployment's result.
///
/// [REST 2024-10-01](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01#remediation)
/// includes `Accepted` in its examples; the [task lifecycle guidance](https://learn.microsoft.com/en-us/azure/governance/policy/concepts/remediation-structure#provisioning-state-and-deployment-summary)
/// additionally documents `Running`.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum PolicyRemediationProvisioningState {
    Accepted,
    Evaluating,
    Running,
    Canceled,
    Cancelling,
    Failed,
    Complete,
    Succeeded,
    #[arbitrary(skip)]
    Other(String),
}

impl From<String> for PolicyRemediationProvisioningState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Accepted" => Self::Accepted,
            "Evaluating" => Self::Evaluating,
            "Running" => Self::Running,
            "Canceled" => Self::Canceled,
            "Cancelling" => Self::Cancelling,
            "Failed" => Self::Failed,
            "Complete" => Self::Complete,
            "Succeeded" => Self::Succeeded,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Other(value)
            }
        }
    }
}

impl From<&PolicyRemediationProvisioningState> for String {
    fn from(value: &PolicyRemediationProvisioningState) -> Self {
        match value {
            PolicyRemediationProvisioningState::Accepted => "Accepted",
            PolicyRemediationProvisioningState::Evaluating => "Evaluating",
            PolicyRemediationProvisioningState::Running => "Running",
            PolicyRemediationProvisioningState::Canceled => "Canceled",
            PolicyRemediationProvisioningState::Cancelling => "Cancelling",
            PolicyRemediationProvisioningState::Failed => "Failed",
            PolicyRemediationProvisioningState::Complete => "Complete",
            PolicyRemediationProvisioningState::Succeeded => "Succeeded",
            PolicyRemediationProvisioningState::Other(value) => value.as_str(),
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(PolicyRemediationProvisioningState);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationProvisioningState);
