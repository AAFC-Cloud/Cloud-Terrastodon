use crate::AuthPartition;
use crate::CacheScope;
use crate::ExecutorKind;
use crate::NonEmptyString;
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
    /// Credential bytes must never participate. Construction and decoding enforce
    /// nonemptiness before ownership can be claimed; a transport owns the format
    /// and coverage of its fingerprint.
    pub fingerprint: NonEmptyString,
}

cloud_terrastodon_registry_values::register_thing!(CacheEntryIdentity);
