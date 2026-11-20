use crate::EntraPrincipalId;
use arbitrary::Arbitrary;
use facet::Facet;

/// A principal identified in an ARM [deny assignment](https://learn.microsoft.com/en-us/rest/api/authorization/deny-assignments/list-for-scope?view=rest-authorization-2022-04-01#principal).
#[derive(Debug, Arbitrary, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDenyAssignmentPrincipal {
    pub id: EntraPrincipalId,
    /// The API schema exposes an open string and includes the system-defined "SystemDefined" principal kind.
    #[facet(rename = "type")]
    pub kind: String,
}

cloud_terrastodon_registry::register_thing!(AzureDenyAssignmentPrincipal);
cloud_terrastodon_registry::register_arbitrary!(AzureDenyAssignmentPrincipal);
