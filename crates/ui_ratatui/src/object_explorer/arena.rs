use super::arena_slot::ArenaSlot;
use super::arena_slot_state::ArenaSlotState;
use super::revision::ArenaRevision;
use super::revision::ArenaRevisions;
use super::revision::RevisionError;
use super::revision::RootRevision;
use super::slot_id::SlotId;
use cloud_terrastodon_registry::RuntimeValue;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ArenaError {
    UnknownSlot(SlotId),
    RootNotReady {
        slot: SlotId,
        state: &'static str,
    },
    Tombstone {
        slot: SlotId,
        previous: &'static str,
    },
    InvalidTransition {
        slot: SlotId,
        from: &'static str,
        to: &'static str,
    },
    Revision(RevisionError),
}

impl fmt::Display for ArenaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSlot(slot) => write!(formatter, "slot {slot} does not exist"),
            Self::RootNotReady { slot, state } => {
                write!(formatter, "slot {slot} is {state}, not Ready")
            }
            Self::Tombstone { slot, previous } => {
                write!(
                    formatter,
                    "slot {slot} is a tombstone (previous state: {previous})"
                )
            }
            Self::InvalidTransition { slot, from, to } => {
                write!(
                    formatter,
                    "slot {slot} cannot transition from {from} to {to}"
                )
            }
            Self::Revision(error) => error.fmt(formatter),
        }
    }
}

impl Error for ArenaError {}

impl From<RevisionError> for ArenaError {
    fn from(value: RevisionError) -> Self {
        Self::Revision(value)
    }
}

/// Instance-owned source of truth for ownership-bearing roots.
#[derive(Default)]
pub(crate) struct Arena {
    next_slot_id: u64,
    slots: BTreeMap<SlotId, ArenaSlot>,
    revisions: ArenaRevisions,
}

impl Arena {
    pub(crate) fn reserve_builder(&mut self) -> Result<SlotId, ArenaError> {
        self.insert_state(ArenaSlotState::Building)
    }

    pub(crate) fn insert_pending(&mut self) -> Result<SlotId, ArenaError> {
        self.insert_state(ArenaSlotState::Pending)
    }

    pub(crate) fn insert_ready(&mut self, value: RuntimeValue) -> Result<SlotId, ArenaError> {
        self.insert_state(ArenaSlotState::Ready(value))
    }

    pub(crate) fn set_ready(
        &mut self,
        slot_id: SlotId,
        value: RuntimeValue,
    ) -> Result<(), ArenaError> {
        let slot = self
            .slots
            .get_mut(&slot_id)
            .ok_or(ArenaError::UnknownSlot(slot_id))?;
        if !matches!(
            slot.state(),
            ArenaSlotState::Building | ArenaSlotState::Pending
        ) {
            return Err(ArenaError::InvalidTransition {
                slot: slot_id,
                from: slot.state().label(),
                to: "Ready",
            });
        }
        slot.replace_state(ArenaSlotState::Ready(value));
        self.revisions.ingest_root_change(slot_id)?;
        Ok(())
    }

    /// Replace one Ready root after an explicit in-place mutation operation.
    ///
    /// Callers must first prove through BorrowGraph that no external borrow
    /// points into the old allocation. Returning the old value keeps its drop
    /// ordered after the arena's linearized revision change.
    pub(crate) fn replace_ready(
        &mut self,
        slot_id: SlotId,
        value: RuntimeValue,
    ) -> Result<RuntimeValue, ArenaError> {
        let slot = self
            .slots
            .get_mut(&slot_id)
            .ok_or(ArenaError::UnknownSlot(slot_id))?;
        if !matches!(slot.state(), ArenaSlotState::Ready(_)) {
            return Err(ArenaError::InvalidTransition {
                slot: slot_id,
                from: slot.state().label(),
                to: "Ready",
            });
        }
        let previous = slot.replace_state(ArenaSlotState::Ready(value));
        let ArenaSlotState::Ready(previous) = previous else {
            unreachable!("Ready transition checked immediately above")
        };
        self.revisions.ingest_root_change(slot_id)?;
        Ok(previous)
    }

    pub(crate) fn delete(&mut self, slot_id: SlotId) -> Result<(), ArenaError> {
        let slot = self
            .slots
            .get_mut(&slot_id)
            .ok_or(ArenaError::UnknownSlot(slot_id))?;
        if !matches!(
            slot.state(),
            ArenaSlotState::Ready(_) | ArenaSlotState::Failed(_) | ArenaSlotState::Cancelled
        ) {
            return Err(ArenaError::InvalidTransition {
                slot: slot_id,
                from: slot.state().label(),
                to: "Tombstone",
            });
        }
        let previous = slot.state().label();
        slot.replace_state(ArenaSlotState::Tombstone { previous });
        self.revisions.ingest_root_change(slot_id)?;
        Ok(())
    }

    /// Tombstone an incomplete root after its engine-owned builder is dropped.
    ///
    /// This is separate from delete so arbitrary callers cannot make a
    /// Building root disappear while BuilderStore still owns construction
    /// state for it.
    pub(crate) fn abandon_builder(&mut self, slot_id: SlotId) -> Result<(), ArenaError> {
        let slot = self
            .slots
            .get_mut(&slot_id)
            .ok_or(ArenaError::UnknownSlot(slot_id))?;
        if !matches!(slot.state(), ArenaSlotState::Building) {
            return Err(ArenaError::InvalidTransition {
                slot: slot_id,
                from: slot.state().label(),
                to: "Tombstone",
            });
        }
        slot.replace_state(ArenaSlotState::Tombstone {
            previous: "Building",
        });
        self.revisions.ingest_root_change(slot_id)?;
        Ok(())
    }

    pub(crate) fn set_failed(
        &mut self,
        slot_id: SlotId,
        message: impl Into<String>,
    ) -> Result<(), ArenaError> {
        let slot = self
            .slots
            .get_mut(&slot_id)
            .ok_or(ArenaError::UnknownSlot(slot_id))?;
        if !matches!(
            slot.state(),
            ArenaSlotState::Building | ArenaSlotState::Pending
        ) {
            return Err(ArenaError::InvalidTransition {
                slot: slot_id,
                from: slot.state().label(),
                to: "Failed",
            });
        }
        slot.replace_state(ArenaSlotState::Failed(message.into()));
        self.revisions.ingest_root_change(slot_id)?;
        Ok(())
    }

    pub(crate) fn cancel_pending(&mut self, slot_id: SlotId) -> Result<(), ArenaError> {
        let slot = self
            .slots
            .get_mut(&slot_id)
            .ok_or(ArenaError::UnknownSlot(slot_id))?;
        if !matches!(slot.state(), ArenaSlotState::Pending) {
            return Err(ArenaError::InvalidTransition {
                slot: slot_id,
                from: slot.state().label(),
                to: "Cancelled",
            });
        }
        slot.replace_state(ArenaSlotState::Cancelled);
        self.revisions.ingest_root_change(slot_id)?;
        Ok(())
    }

    pub(crate) fn consume(&mut self, slot_id: SlotId) -> Result<RuntimeValue, ArenaError> {
        let slot = self
            .slots
            .get_mut(&slot_id)
            .ok_or(ArenaError::UnknownSlot(slot_id))?;
        if !matches!(slot.state(), ArenaSlotState::Ready(_)) {
            return Err(ArenaError::InvalidTransition {
                slot: slot_id,
                from: slot.state().label(),
                to: "Consumed",
            });
        }
        let previous = slot.replace_state(ArenaSlotState::Consumed);
        let ArenaSlotState::Ready(value) = previous else {
            unreachable!("Ready transition checked immediately above")
        };
        self.revisions.ingest_root_change(slot_id)?;
        Ok(value)
    }

    pub(crate) fn slot(&self, slot_id: SlotId) -> Option<&ArenaSlot> {
        self.slots.get(&slot_id)
    }

    pub(crate) fn ready_value(&self, slot_id: SlotId) -> Option<&RuntimeValue> {
        self.resolve_root(slot_id).ok()
    }

    pub(crate) fn resolve_root(&self, slot_id: SlotId) -> Result<&RuntimeValue, ArenaError> {
        let slot = self
            .slots
            .get(&slot_id)
            .ok_or(ArenaError::UnknownSlot(slot_id))?;
        match slot.state() {
            ArenaSlotState::Ready(value) => Ok(value),
            ArenaSlotState::Tombstone { previous } => Err(ArenaError::Tombstone {
                slot: slot_id,
                previous,
            }),
            state => Err(ArenaError::RootNotReady {
                slot: slot_id,
                state: state.label(),
            }),
        }
    }

    pub(crate) fn ready_slot_ids(&self) -> impl Iterator<Item = SlotId> + '_ {
        self.slots
            .values()
            .filter_map(|slot| slot.state().ready_value().is_some().then_some(slot.id()))
    }

    /// Ownership-bearing roots that should remain addressable in the object
    /// pool, including builders and pending/terminal lifecycle states.
    ///
    /// Tombstones remain valid stale identities but are not visible cards.
    pub(crate) fn object_pool_slot_ids(&self) -> impl Iterator<Item = SlotId> + '_ {
        self.slots.values().filter_map(|slot| {
            (!matches!(slot.state(), ArenaSlotState::Tombstone { .. })).then_some(slot.id())
        })
    }

    pub(crate) fn allocated_slot_count(&self) -> usize {
        self.slots.len()
    }

    pub(crate) const fn arena_revision(&self) -> ArenaRevision {
        self.revisions.arena_revision()
    }

    pub(crate) fn root_revision(&self, slot_id: SlotId) -> Option<RootRevision> {
        self.revisions.root_revision(slot_id)
    }

    fn insert_state(&mut self, state: ArenaSlotState) -> Result<SlotId, ArenaError> {
        let slot_id = SlotId::new(self.next_slot_id);
        self.next_slot_id = self
            .next_slot_id
            .checked_add(1)
            .expect("arena SlotId space exhausted");
        self.revisions.ingest_root_insert(slot_id)?;
        self.slots.insert(slot_id, ArenaSlot::new(slot_id, state));
        Ok(slot_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::arena_address_source::ArenaAddressSource;
    use crate::object_explorer::borrow_graph::BorrowGraph;
    use crate::object_explorer::breadcrumbs::Breadcrumbs;
    use crate::object_explorer::tab::Tab;
    use crate::object_explorer::value_address::ValueAddress;
    use crate::object_explorer::value_path::ValuePathSegment;
    use facet::Facet;
    use std::borrow::Cow;

    #[derive(Clone, Debug, facet::Facet)]
    #[repr(C)]
    struct MyThing {
        age: usize,
        name: String,
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: facet::Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    #[test]
    fn two_arenas_do_not_share_same_numeric_slot_id() {
        let mut first = Arena::default();
        let mut second = Arena::default();

        let first_id = first.insert_ready(runtime(String::from("first"))).unwrap();
        let second_id = second
            .insert_ready(runtime(String::from("second")))
            .unwrap();

        assert_eq!(first_id, second_id);
        assert_ne!(
            first.ready_value(first_id).unwrap().peek().as_str(),
            second.ready_value(second_id).unwrap().peek().as_str()
        );
    }

    #[test]
    fn ready_roots_follow_slot_id_and_non_ready_states_are_not_values() {
        let mut arena = Arena::default();
        let first = arena
            .insert_ready(runtime(MyThing {
                age: 42,
                name: "Ada".to_owned(),
            }))
            .unwrap();
        let building = arena.reserve_builder().unwrap();
        let last = arena.insert_ready(runtime(String::from("last"))).unwrap();

        assert_eq!(arena.ready_slot_ids().collect::<Vec<_>>(), [first, last]);
        assert!(arena.ready_value(building).is_none());
    }

    #[test]
    fn materialization_and_deletion_advance_only_the_addressed_root() {
        let mut arena = Arena::default();
        let building = arena.reserve_builder().unwrap();
        let stable = arena.insert_ready(runtime(String::from("stable"))).unwrap();
        let building_before = arena.root_revision(building).unwrap();
        let stable_before = arena.root_revision(stable).unwrap();
        let arena_before = arena.arena_revision();

        arena
            .set_ready(building, runtime(String::from("ready")))
            .unwrap();

        assert!(arena.arena_revision() > arena_before);
        assert!(arena.root_revision(building).unwrap() > building_before);
        assert_eq!(arena.root_revision(stable), Some(stable_before));
        assert_eq!(
            arena.ready_slot_ids().collect::<Vec<_>>(),
            [building, stable]
        );

        arena.delete(building).unwrap();
        assert_eq!(arena.ready_slot_ids().collect::<Vec<_>>(), [stable]);
        assert!(matches!(
            arena.slot(building).map(ArenaSlot::state),
            Some(ArenaSlotState::Tombstone { .. })
        ));
    }

    #[test]
    fn ready_values_cannot_be_silently_replaced() {
        let mut arena = Arena::default();
        let ready = arena
            .insert_ready(runtime(String::from("original")))
            .unwrap();

        let error = arena
            .set_ready(ready, runtime(String::from("replacement")))
            .expect_err("Ready replacement needs an explicit mutation API");

        assert!(matches!(
            error,
            ArenaError::InvalidTransition {
                slot,
                from: "Ready",
                to: "Ready",
            } if slot == ready
        ));
    }

    #[test]
    fn pending_cancel_failure_success_and_ready_consumption_follow_the_lifecycle() {
        let mut arena = Arena::default();

        let success = arena.insert_pending().unwrap();
        arena
            .set_ready(success, runtime(String::from("completed")))
            .unwrap();
        assert_eq!(
            arena.resolve_root(success).unwrap().peek().as_str(),
            Some("completed")
        );

        let failure = arena.insert_pending().unwrap();
        arena.set_failed(failure, "network failed").unwrap();
        assert!(matches!(
            arena.slot(failure).map(ArenaSlot::state),
            Some(ArenaSlotState::Failed(message)) if message == "network failed"
        ));

        let cancelled = arena.insert_pending().unwrap();
        arena.cancel_pending(cancelled).unwrap();
        assert!(matches!(
            arena.slot(cancelled).map(ArenaSlot::state),
            Some(ArenaSlotState::Cancelled)
        ));

        arena.delete(success).unwrap();
        arena.delete(failure).unwrap();
        arena.delete(cancelled).unwrap();

        let consumed = arena
            .insert_ready(runtime(String::from("moved into invocation")))
            .unwrap();
        let value = arena.consume(consumed).unwrap();
        assert_eq!(value.peek().as_str(), Some("moved into invocation"));
        assert!(matches!(
            arena.slot(consumed).map(ArenaSlot::state),
            Some(ArenaSlotState::Consumed)
        ));
        assert!(matches!(
            arena.delete(consumed),
            Err(ArenaError::InvalidTransition {
                slot,
                from: "Consumed",
                to: "Tombstone",
            }) if slot == consumed
        ));
    }

    #[test]
    fn root_resolution_and_illegal_transitions_return_typed_errors() {
        let mut arena = Arena::default();
        let building = arena.reserve_builder().unwrap();
        assert!(matches!(
            arena.resolve_root(building),
            Err(ArenaError::RootNotReady {
                slot,
                state: "Building",
            }) if slot == building
        ));
        assert!(matches!(
            arena.delete(building),
            Err(ArenaError::InvalidTransition {
                slot,
                from: "Building",
                to: "Tombstone",
            }) if slot == building
        ));

        let unknown = SlotId::new(10_000);
        assert!(matches!(
            arena.resolve_root(unknown),
            Err(ArenaError::UnknownSlot(slot)) if slot == unknown
        ));

        arena
            .set_ready(building, runtime(String::from("ready")))
            .unwrap();
        arena.delete(building).unwrap();
        assert!(matches!(
            arena.resolve_root(building),
            Err(ArenaError::Tombstone {
                slot,
                previous: "Ready",
            }) if slot == building
        ));
        assert!(matches!(
            arena.set_ready(building, runtime(String::from("stale"))),
            Err(ArenaError::InvalidTransition {
                slot,
                from: "Tombstone",
                to: "Ready",
            }) if slot == building
        ));
        assert!(matches!(
            arena.delete(building),
            Err(ArenaError::InvalidTransition {
                slot,
                from: "Tombstone",
                to: "Tombstone",
            }) if slot == building
        ));

        let pending = arena.insert_pending().unwrap();
        assert!(matches!(
            arena.delete(pending),
            Err(ArenaError::InvalidTransition {
                slot,
                from: "Pending",
                to: "Tombstone",
            }) if slot == pending
        ));

        let next = arena.insert_ready(runtime(String::from("new"))).unwrap();
        assert!(next > building, "tombstoned SlotIds must never be reused");
    }

    #[test]
    fn tab_uses_ordinary_arena_reflection_borrow_and_deletion_paths() {
        let mut arena = Arena::default();
        let tab = arena
            .insert_ready(runtime(Tab::new("admins", Breadcrumbs::default())))
            .unwrap();
        assert!(
            arena
                .resolve_root(tab)
                .unwrap()
                .shape()
                .is_shape(Tab::SHAPE)
        );

        let breadcrumbs =
            ValueAddress::root(tab).child(ValuePathSegment::Field("breadcrumbs".to_owned()));
        assert!(
            ArenaAddressSource::new(&arena)
                .resolve(&breadcrumbs)
                .is_ok_and(|value| value.shape().is_shape(Breadcrumbs::SHAPE))
        );

        let borrower = arena.reserve_builder().unwrap();
        let mut graph = BorrowGraph::default();
        let lease = graph
            .borrow(&arena, ValueAddress::root(tab), borrower, "tab")
            .unwrap();
        let borrowed = RuntimeValue::from_borrowed_pointer(
            <Cow<'static, Tab>>::SHAPE,
            arena.resolve_root(tab).unwrap().peek(),
        )
        .expect("Tab follows the ordinary reflected Cow pointer path");
        assert!(graph.protects_root(tab));
        drop(borrowed);
        graph.release(lease).unwrap();

        arena.delete(tab).unwrap();
        assert!(matches!(
            arena.resolve_root(tab),
            Err(ArenaError::Tombstone {
                slot,
                previous: "Ready",
            }) if slot == tab
        ));
    }
}
