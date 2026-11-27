use eyre::Report;
use eyre::eyre;
use facet::Facet;

/// A failed [Microsoft Graph membership lookup](https://learn.microsoft.com/graph/api/directoryobject-getmembergroups?view=graph-rest-1.0)
/// that leaves an Azure role assignment report with direct assignments only.
/// Consumers can display the original error report. Serialized output carries its plain error chain.
#[derive(Debug, Facet)]
#[facet(opaque, proxy = String)]
pub struct EntraUserAzureRoleMembershipFailure {
    pub report: Report,
}

impl From<&EntraUserAzureRoleMembershipFailure> for String {
    fn from(failure: &EntraUserAzureRoleMembershipFailure) -> Self {
        format!("{:#}", failure.report)
    }
}

impl From<String> for EntraUserAzureRoleMembershipFailure {
    fn from(message: String) -> Self {
        Self {
            report: eyre!(message),
        }
    }
}

cloud_terrastodon_registry::register_thing!(EntraUserAzureRoleMembershipFailure);
