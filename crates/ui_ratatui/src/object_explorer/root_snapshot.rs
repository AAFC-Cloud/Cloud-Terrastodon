use super::arena::{Arena, ArenaError};
use super::arena_address_source::ArenaAddressSource;
use super::arena_slot_state::ArenaSlotState;
use super::builder_snapshot::{BuilderKindSnapshot, BuilderSnapshot};
use super::card_row_key::CardRowKey;
use super::card_row_snapshot::{CardRowContent, CardRowSnapshot};
use super::card_snapshot::CardSnapshot;
use super::revision::RootRevision;
use super::slot_id::SlotId;
use super::value_address::ValueAddress;
use super::value_builder::BuilderStore;
use super::value_resolution_error::ValueResolutionError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RootLifecycleSnapshot {
    Building,
    Ready,
    Pending,
    Failed(String),
    Cancelled,
    Consumed,
    Tombstone { previous: &'static str },
}

/// Bounded observation of one ownership-bearing root.
///
/// Ready values reuse the ordinary reflected CardSnapshot. Non-ready values
/// expose lifecycle and builder metadata only; they never masquerade as query
/// results and never require a synthetic view slot.
#[derive(Clone, Debug)]
pub(crate) struct RootSnapshot {
    slot: SlotId,
    revision: RootRevision,
    lifecycle: RootLifecycleSnapshot,
    builder: Option<BuilderSnapshot>,
    card: CardSnapshot,
}

impl RootSnapshot {
    pub(crate) fn observe(
        arena: &Arena,
        builders: &BuilderStore,
        slot: SlotId,
        max_relationship_rows: usize,
    ) -> Result<Self, RootSnapshotError> {
        let arena_slot = arena.slot(slot).ok_or(ArenaError::UnknownSlot(slot))?;
        let revision = arena
            .root_revision(slot)
            .ok_or(ArenaError::UnknownSlot(slot))?;
        let address = ValueAddress::root(slot);
        let (lifecycle, builder, card) = match arena_slot.state() {
            ArenaSlotState::Ready(_) => (
                RootLifecycleSnapshot::Ready,
                None,
                CardSnapshot::observe(
                    &ArenaAddressSource::new(arena),
                    address,
                    max_relationship_rows,
                )?,
            ),
            ArenaSlotState::Building => {
                let builder = builders
                    .builder(slot)
                    .map_or_else(BuilderSnapshot::shape_unset, |builder| {
                        BuilderSnapshot::observe(builder, max_relationship_rows)
                    });
                let mut rows = vec![CardRowSnapshot::new(
                    CardRowKey::Shape,
                    "shape",
                    CardRowContent::Text(builder.shape_name().unwrap_or("unset").to_owned()),
                )];
                match builder.kind() {
                    BuilderKindSnapshot::ShapeUnset => {}
                    BuilderKindSnapshot::Scalar { value_is_set } => {
                        rows.push(CardRowSnapshot::new(
                            CardRowKey::Value,
                            "value",
                            CardRowContent::Text(
                                if *value_is_set { "set" } else { "unset" }.to_owned(),
                            ),
                        ))
                    }
                    BuilderKindSnapshot::Struct => {}
                    BuilderKindSnapshot::Enum {
                        selected_variant_name,
                        ..
                    } => rows.push(CardRowSnapshot::new(
                        CardRowKey::Variant,
                        "variant",
                        CardRowContent::Text(
                            selected_variant_name
                                .as_deref()
                                .unwrap_or("unset")
                                .to_owned(),
                        ),
                    )),
                }
                rows.extend(builder.fields().iter().map(|field| {
                    CardRowSnapshot::new(
                        CardRowKey::Field(field.name().to_owned()),
                        field.name(),
                        CardRowContent::Binding(field.binding().clone()),
                    )
                    .with_type_name(field.shape_name())
                }));
                rows.push(CardRowSnapshot::new(
                    CardRowKey::Status,
                    "status",
                    CardRowContent::Text("Building".to_owned()),
                ));
                let card = CardSnapshot::owned(
                    slot,
                    builder.shape_name().unwrap_or("shape unset"),
                    revision,
                    rows,
                    builder.fields_complete(),
                );
                (RootLifecycleSnapshot::Building, Some(builder), card)
            }
            ArenaSlotState::Pending => lifecycle_card(
                slot,
                revision,
                RootLifecycleSnapshot::Pending,
                "Pending",
                None,
            ),
            ArenaSlotState::Failed(message) => lifecycle_card(
                slot,
                revision,
                RootLifecycleSnapshot::Failed(message.clone()),
                "Failed",
                Some(message.clone()),
            ),
            ArenaSlotState::Cancelled => lifecycle_card(
                slot,
                revision,
                RootLifecycleSnapshot::Cancelled,
                "Cancelled",
                None,
            ),
            ArenaSlotState::Consumed => lifecycle_card(
                slot,
                revision,
                RootLifecycleSnapshot::Consumed,
                "Consumed",
                None,
            ),
            ArenaSlotState::Tombstone { previous } => lifecycle_card(
                slot,
                revision,
                RootLifecycleSnapshot::Tombstone { previous },
                "Tombstone",
                Some(format!("previous state: {previous}")),
            ),
        };
        Ok(Self {
            slot,
            revision,
            lifecycle,
            builder,
            card,
        })
    }

    pub(crate) const fn slot(&self) -> SlotId {
        self.slot
    }

    pub(crate) const fn revision(&self) -> RootRevision {
        self.revision
    }

    pub(crate) const fn lifecycle(&self) -> &RootLifecycleSnapshot {
        &self.lifecycle
    }

    pub(crate) const fn builder(&self) -> Option<&BuilderSnapshot> {
        self.builder.as_ref()
    }

    pub(crate) const fn card(&self) -> &CardSnapshot {
        &self.card
    }
}

fn lifecycle_card(
    slot: SlotId,
    revision: RootRevision,
    lifecycle: RootLifecycleSnapshot,
    label: &'static str,
    detail: Option<String>,
) -> (RootLifecycleSnapshot, Option<BuilderSnapshot>, CardSnapshot) {
    let mut rows = vec![CardRowSnapshot::new(
        CardRowKey::Status,
        "status",
        CardRowContent::Text(label.to_owned()),
    )];
    if let Some(detail) = detail {
        rows.push(CardRowSnapshot::new(
            CardRowKey::Value,
            "detail",
            CardRowContent::Text(detail),
        ));
    }
    (
        lifecycle,
        None,
        CardSnapshot::owned(slot, label, revision, rows, true),
    )
}

#[derive(Debug)]
pub(crate) enum RootSnapshotError {
    Arena(ArenaError),
    Resolve(ValueResolutionError),
}

impl std::fmt::Display for RootSnapshotError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arena(error) => error.fmt(formatter),
            Self::Resolve(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for RootSnapshotError {}

impl From<ArenaError> for RootSnapshotError {
    fn from(value: ArenaError) -> Self {
        Self::Arena(value)
    }
}

impl From<ValueResolutionError> for RootSnapshotError {
    fn from(value: ValueResolutionError) -> Self {
        Self::Resolve(value)
    }
}

#[cfg(test)]
mod tests {
    use facet::Facet;

    use super::*;
    use crate::object_explorer::borrow_graph::BorrowGraph;

    #[derive(Facet)]
    #[repr(C)]
    struct BuildThing {
        name: String,
        count: usize,
    }

    #[test]
    fn shape_unset_and_struct_builders_are_observed_without_ready_values() {
        let mut arena = Arena::default();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        let unset = builders.reserve(&mut arena).unwrap();
        let unset_snapshot = RootSnapshot::observe(&arena, &builders, unset, 1).unwrap();
        assert_eq!(unset_snapshot.lifecycle(), &RootLifecycleSnapshot::Building);
        assert_eq!(
            unset_snapshot.builder().unwrap().kind(),
            &BuilderKindSnapshot::ShapeUnset
        );
        assert!(arena.ready_value(unset).is_none());

        let (defined, transition) = builders
            .create_and_finalize(&mut arena, &mut borrows, BuildThing::SHAPE)
            .unwrap();
        assert_eq!(
            transition,
            crate::object_explorer::value_builder::BuilderTransition::Building
        );
        let defined_snapshot = RootSnapshot::observe(&arena, &builders, defined, 1).unwrap();
        let builder = defined_snapshot.builder().unwrap();
        assert_eq!(builder.shape_name(), Some("BuildThing"));
        assert_eq!(builder.fields().len(), 1);
        assert_eq!(builder.fields()[0].name(), "name");
        assert!(!builder.fields_complete());
        assert!(!defined_snapshot.card().relationships_complete());
        assert!(arena.ready_value(defined).is_none());
    }

    #[test]
    fn pending_and_failed_roots_have_lifecycle_cards_without_reflection() {
        let mut arena = Arena::default();
        let builders = BuilderStore::default();
        let pending = arena.insert_pending().unwrap();
        let pending_snapshot = RootSnapshot::observe(&arena, &builders, pending, 0).unwrap();
        assert_eq!(
            pending_snapshot.lifecycle(),
            &RootLifecycleSnapshot::Pending
        );
        assert_eq!(pending_snapshot.card().owned_slot(), Some(pending));

        arena.set_failed(pending, "offline".to_owned()).unwrap();
        let failed_snapshot = RootSnapshot::observe(&arena, &builders, pending, 0).unwrap();
        assert_eq!(
            failed_snapshot.lifecycle(),
            &RootLifecycleSnapshot::Failed("offline".to_owned())
        );
        assert!(
            failed_snapshot
                .card()
                .rows()
                .iter()
                .any(|row| { row.content() == &CardRowContent::Text("offline".to_owned()) })
        );
    }
}
