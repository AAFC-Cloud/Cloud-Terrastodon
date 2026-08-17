use super::slot_id::SlotId;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ArenaRevision(u64);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct RootRevision(u64);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct QueryRevision(u64);

impl QueryRevision {
    pub(crate) fn next(self) -> Self {
        Self(
            self.0
                .checked_add(1)
                .expect("query revision space exhausted"),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScanRevisionStamp {
    pub(crate) arena: ArenaRevision,
    pub(crate) query: QueryRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RevisionError {
    DuplicateRoot(SlotId),
    UnknownRoot(SlotId),
}

impl fmt::Display for RevisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateRoot(slot_id) => write!(formatter, "slot {slot_id} already exists"),
            Self::UnknownRoot(slot_id) => write!(formatter, "slot {slot_id} does not exist"),
        }
    }
}

impl Error for RevisionError {}

/// Revision index owned by the single-writer explorer engine.
///
/// Calls are named `ingest_*` to make the linearization point explicit:
/// background work does not alter revisions until the engine accepts its
/// completion command.
#[derive(Debug, Default)]
pub(crate) struct ArenaRevisions {
    arena: ArenaRevision,
    roots: BTreeMap<SlotId, RootRevision>,
}

impl ArenaRevisions {
    pub(crate) const fn arena_revision(&self) -> ArenaRevision {
        self.arena
    }

    pub(crate) fn root_revision(&self, slot_id: SlotId) -> Option<RootRevision> {
        self.roots.get(&slot_id).copied()
    }

    pub(crate) fn ingest_root_insert(
        &mut self,
        slot_id: SlotId,
    ) -> Result<RootRevision, RevisionError> {
        if self.roots.contains_key(&slot_id) {
            return Err(RevisionError::DuplicateRoot(slot_id));
        }
        self.advance_arena();
        let revision = RootRevision(1);
        self.roots.insert(slot_id, revision);
        Ok(revision)
    }

    pub(crate) fn ingest_root_change(
        &mut self,
        slot_id: SlotId,
    ) -> Result<RootRevision, RevisionError> {
        let revision = self
            .roots
            .get_mut(&slot_id)
            .ok_or(RevisionError::UnknownRoot(slot_id))?;
        revision.0 = revision
            .0
            .checked_add(1)
            .expect("root revision space exhausted");
        let result = *revision;
        self.advance_arena();
        Ok(result)
    }

    pub(crate) fn cache_is_current(&self, slot_id: SlotId, observed: RootRevision) -> bool {
        self.root_revision(slot_id) == Some(observed)
    }

    pub(crate) const fn scan_stamp(&self, query: QueryRevision) -> ScanRevisionStamp {
        ScanRevisionStamp {
            arena: self.arena,
            query,
        }
    }

    fn advance_arena(&mut self) {
        self.arena.0 = self
            .arena
            .0
            .checked_add(1)
            .expect("arena revision space exhausted");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::value_address::ValueAddress;
    use crate::object_explorer::value_path::ValuePathSegment;

    #[test]
    fn unrelated_root_change_preserves_selection_and_root_cache() {
        let selected_root = SlotId::new(3);
        let unrelated_root = SlotId::new(8);
        let selection =
            ValueAddress::root(selected_root).child(ValuePathSegment::Field("name".to_owned()));
        let mut revisions = ArenaRevisions::default();
        let selected_revision = revisions.ingest_root_insert(selected_root).unwrap();
        revisions.ingest_root_insert(unrelated_root).unwrap();

        revisions.ingest_root_change(unrelated_root).unwrap();

        assert_eq!(selection.root_id(), selected_root);
        assert!(revisions.cache_is_current(selected_root, selected_revision));
    }

    #[test]
    fn changed_root_invalidates_only_its_cache() {
        let changed_root = SlotId::new(2);
        let stable_root = SlotId::new(4);
        let mut revisions = ArenaRevisions::default();
        let changed_revision = revisions.ingest_root_insert(changed_root).unwrap();
        let stable_revision = revisions.ingest_root_insert(stable_root).unwrap();

        revisions.ingest_root_change(changed_root).unwrap();

        assert!(!revisions.cache_is_current(changed_root, changed_revision));
        assert!(revisions.cache_is_current(stable_root, stable_revision));
    }

    #[test]
    fn aggregate_scan_uses_arena_and_query_revision_stamp() {
        let mut revisions = ArenaRevisions::default();
        let root = SlotId::new(1);
        revisions.ingest_root_insert(root).unwrap();
        let query_revision = QueryRevision::default();
        let stamp = revisions.scan_stamp(query_revision);

        assert_eq!(stamp, revisions.scan_stamp(query_revision));
        assert_ne!(stamp, revisions.scan_stamp(query_revision.next()));

        revisions.ingest_root_change(root).unwrap();
        assert_ne!(stamp, revisions.scan_stamp(query_revision));
    }

    #[test]
    fn background_completion_revises_arena_only_when_ingested() {
        let output_slot = SlotId::new(12);
        let mut revisions = ArenaRevisions::default();
        revisions.ingest_root_insert(output_slot).unwrap();
        let while_future_is_ready_but_not_ingested = revisions.arena_revision();

        // A completed future is external data until ExplorerEngine handles its
        // completion command; merely holding it cannot mutate this index.
        let completed_background_value = String::from("ready");
        assert_eq!(
            revisions.arena_revision(),
            while_future_is_ready_but_not_ingested
        );

        revisions.ingest_root_change(output_slot).unwrap();
        drop(completed_background_value);
        assert_ne!(
            revisions.arena_revision(),
            while_future_is_ready_but_not_ingested
        );
    }

    #[test]
    fn tombstoning_a_root_advances_its_revision_without_reusing_identity() {
        let root = SlotId::new(7);
        let mut revisions = ArenaRevisions::default();
        let root_revision = revisions.ingest_root_insert(root).unwrap();
        let before_tombstone = revisions.arena_revision();

        revisions.ingest_root_change(root).unwrap();

        assert!(revisions.arena_revision() > before_tombstone);
        assert!(!revisions.cache_is_current(root, root_revision));
        assert!(revisions.root_revision(root).is_some());
    }
}
