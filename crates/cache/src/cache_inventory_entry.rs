use crate::CacheEntryIdentity;
use crate::CacheInventorySelection;
use facet::Facet;

/// Inspectable cache address and metadata for its currently selected artifact.
///
/// This snapshot does not load request inputs or response payloads. An entry with
/// no selected artifact may retain failed, invalidated, or interrupted attempts.
#[derive(Clone, Debug, Facet)]
pub struct CacheInventoryEntry {
    pub identity: CacheEntryIdentity,
    pub selection: CacheInventorySelection,
}
