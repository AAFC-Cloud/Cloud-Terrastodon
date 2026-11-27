use arbitrary::Arbitrary;
use facet::Facet;

/// Opaque Microsoft Graph [appRoleAssignment ID](https://learn.microsoft.com/graph/api/resources/approleassignment?view=graph-rest-1.0#properties).
#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[facet(transparent)]
pub struct EntraAppRoleAssignmentId(pub String);

cloud_terrastodon_registry::register_thing!(EntraAppRoleAssignmentId);
cloud_terrastodon_registry::register_arbitrary!(EntraAppRoleAssignmentId);
