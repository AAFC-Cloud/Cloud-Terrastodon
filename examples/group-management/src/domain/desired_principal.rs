use cloud_terrastodon::azure_types::EntraPrincipalId;
use cloud_terrastodon::azure_types::UserPrincipalName;

/// A pinned object ID, a user principal name to resolve, or both.
///
/// IDs are authoritative; names provide readable annotations and drift checks.
/// Group and service-principal references normally use an ID and display name.
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesiredPrincipal {
    #[facet(default)]
    pub id: Option<EntraPrincipalId>,
    #[facet(default)]
    pub user_principal_name: Option<UserPrincipalName>,
    #[facet(default)]
    pub display_name: Option<String>,
}

impl DesiredPrincipal {
    /// Match a lookup independently of its optional human-readable annotation.
    pub fn same_identity(&self, other: &Self) -> bool {
        self.id == other.id
            && match (&self.user_principal_name, &other.user_principal_name) {
                (Some(left), Some(right)) => left.matches(right),
                (None, None) => true,
                _ => false,
            }
    }
}
