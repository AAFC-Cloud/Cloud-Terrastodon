use crate::AuthPartition;
use crate::CacheScope;
use crate::ExecutorKind;
use facet::Facet;

/// A concrete artifact entry within an already claimed operation namespace.
///
/// Ownership is checked at `(kind, scope)` before selecting a partition or
/// fingerprint. These variants must not hide conflicting operation/output
/// contracts. Physical storage validates the complete exact identity before
/// accepting an entry; diagnostic type names are not part of this identity.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Facet)]
pub struct CacheEntryIdentity {
    pub kind: ExecutorKind,
    pub scope: CacheScope,
    pub auth_partition: AuthPartition,
    /// Nonempty, non-secret fingerprint of all semantic execution inputs.
    ///
    /// Credential bytes must never participate. The store checks nonemptiness
    /// when admitting an entry; a transport owns its fingerprint construction.
    pub fingerprint: String,
}

cloud_terrastodon_registry_values::register_thing!(CacheEntryIdentity);
