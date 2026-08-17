use facet::Type;

use super::arena_address_source::ArenaAddressSource;
use super::card_address::CardAddress;
use super::card_row_key::CardRowKey;
use super::card_row_snapshot::{CardRowContent, CardRowSnapshot};
use super::resolved_value::ResolvedValue;
use super::revision::RootRevision;
use super::root_action_snapshot::RootActionSnapshot;
use super::slot_id::SlotId;
use super::value_address::ValueAddress;
use super::value_path::ValuePathSegment;
use super::value_resolution_error::ValueResolutionError;

const MAX_SCALAR_SUMMARY_CHARS: usize = 256;

/// UI-neutral description of one reflected card.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CardSnapshot {
    address: CardAddress,
    owned_slot: Option<SlotId>,
    shape: String,
    root_revision: RootRevision,
    rows: Vec<CardRowSnapshot>,
    relationships_complete: bool,
}

impl CardSnapshot {
    pub(crate) fn new_slot() -> Self {
        Self {
            address: CardAddress::NewSlot,
            owned_slot: None,
            shape: "new object".to_owned(),
            root_revision: RootRevision::default(),
            rows: Vec::new(),
            relationships_complete: true,
        }
    }

    pub(crate) fn owned(
        slot: SlotId,
        shape: impl Into<String>,
        root_revision: RootRevision,
        rows: Vec<CardRowSnapshot>,
        relationships_complete: bool,
    ) -> Self {
        Self {
            address: CardAddress::Value(ValueAddress::root(slot)),
            owned_slot: Some(slot),
            shape: shape.into(),
            root_revision,
            rows,
            relationships_complete,
        }
    }

    /// Observe one card and at most max_relationship_rows direct children.
    pub(crate) fn observe(
        source: &ArenaAddressSource<'_>,
        address: ValueAddress,
        max_relationship_rows: usize,
    ) -> Result<Self, ValueResolutionError> {
        let resolved = source.resolve(&address)?;
        let shape = cloud_terrastodon_registry::describe_shape(resolved.shape()).to_owned();
        let mut rows = vec![CardRowSnapshot::new(
            CardRowKey::Shape,
            "shape",
            CardRowContent::Text(shape.clone()),
        )];

        let mut relationships_complete = true;
        let mut admitted_relationships = 0;
        match source.reflected_children(&address)? {
            Some(mut children) => {
                for child in children.by_ref().take(max_relationship_rows) {
                    rows.push(relationship_row(source, child)?);
                    admitted_relationships += 1;
                }
                relationships_complete = children.next().is_none();
            }
            None => {
                if let Some(summary) = scalar_summary(resolved) {
                    rows.push(CardRowSnapshot::new(
                        CardRowKey::Value,
                        "value",
                        CardRowContent::Text(summary),
                    ));
                }
            }
        }

        let owned_slot = address
            .path()
            .segments()
            .is_empty()
            .then_some(address.root_id());
        if owned_slot.is_some() {
            let actions = cloud_terrastodon_registry::functions_from(resolved.shape())
                .into_iter()
                .flat_map(|function| {
                    [
                        Some(RootActionSnapshot::Invoke {
                            function,
                            mode: super::invocation_mode::InvocationMode::Consume,
                        }),
                        Some(RootActionSnapshot::Invoke {
                            function,
                            mode: super::invocation_mode::InvocationMode::Retain,
                        }),
                        super::production_controller::arbitrary_constructor_for(
                            function.output_shape,
                        )
                        .map(|constructor| {
                            RootActionSnapshot::InvokeArbitrary {
                                request_function: function,
                                constructor,
                            }
                        }),
                    ]
                    .into_iter()
                    .flatten()
                });
            let mut actions = actions.peekable();
            for action in actions
                .by_ref()
                .take(max_relationship_rows.saturating_sub(admitted_relationships))
            {
                rows.push(CardRowSnapshot::new(
                    CardRowKey::Action(action.id()),
                    "action",
                    super::card_row_snapshot::CardRowContent::RootAction(action),
                ));
            }
            relationships_complete &= actions.next().is_none();
        }
        Ok(Self {
            address: CardAddress::Value(address),
            owned_slot,
            shape,
            root_revision: resolved.root_revision(),
            rows,
            relationships_complete,
        })
    }

    pub(crate) const fn address(&self) -> &CardAddress {
        &self.address
    }

    pub(crate) const fn owned_slot(&self) -> Option<SlotId> {
        self.owned_slot
    }

    pub(crate) fn shape(&self) -> &str {
        &self.shape
    }

    pub(crate) const fn root_revision(&self) -> RootRevision {
        self.root_revision
    }

    pub(crate) fn rows(&self) -> &[CardRowSnapshot] {
        &self.rows
    }

    pub(crate) const fn relationships_complete(&self) -> bool {
        self.relationships_complete
    }
}

fn relationship_row(
    source: &ArenaAddressSource<'_>,
    address: ValueAddress,
) -> Result<CardRowSnapshot, ValueResolutionError> {
    let resolved = source.resolve(&address)?;
    let type_name = cloud_terrastodon_registry::describe_shape(resolved.shape()).to_owned();
    let segment = address
        .path()
        .segments()
        .last()
        .expect("a reflected child has a non-empty path");
    let (key, label) = match segment {
        ValuePathSegment::Field(field) => (CardRowKey::Field(field.clone()), field.clone()),
        ValuePathSegment::Index(index) => (CardRowKey::Element(*index), format!("[{index}]")),
        ValuePathSegment::Key(key) => (CardRowKey::MapValue(key.clone()), format!("[{key:?}]")),
    };
    let row = CardRowSnapshot::new(key, label, CardRowContent::Address(address))
        .with_type_name(type_name);
    Ok(match scalar_summary(resolved) {
        Some(value) => row.with_value_display(value),
        None => row,
    })
}

fn scalar_summary(value: ResolvedValue<'_>) -> Option<String> {
    let value = value.peek().innermost_peek();
    if let Some(text) = value.as_str() {
        return Some(truncate(text, MAX_SCALAR_SUMMARY_CHARS));
    }
    if matches!(value.shape().ty, Type::Primitive(_)) {
        return Some(truncate(&value.to_string(), MAX_SCALAR_SUMMARY_CHARS));
    }
    None
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let mut result = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        result.push('…');
    }
    result
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;

    use super::*;
    use crate::object_explorer::arena::Arena;
    use crate::object_explorer::card_window::CardWindow;

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct MyThing {
        age: usize,
        name: String,
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    #[test]
    fn my_thing_is_one_owned_card_plus_addressed_field_cards() {
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime(MyThing {
                age: 42,
                name: "Ada".to_owned(),
            }))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let window = CardWindow::first(&source, NonZeroUsize::new(3).unwrap(), 8).unwrap();
        let age = ValueAddress::root(root).child(ValuePathSegment::Field("age".to_owned()));
        let name = ValueAddress::root(root).child(ValuePathSegment::Field("name".to_owned()));

        assert_eq!(window.cards().len(), 3);
        assert!(!window.has_before());
        assert!(!window.has_after());

        let root_card = &window.cards()[0];
        assert_eq!(
            root_card.address(),
            &CardAddress::Value(ValueAddress::root(root))
        );
        assert_eq!(root_card.owned_slot(), Some(root));
        assert_eq!(root_card.shape(), "MyThing");
        assert!(root_card.relationships_complete());
        assert!(root_card.rows().iter().any(|row| {
            row.key() == &CardRowKey::Field("age".to_owned())
                && row.content() == &CardRowContent::Address(age.clone())
        }));
        assert!(root_card.rows().iter().any(|row| {
            row.key() == &CardRowKey::Field("name".to_owned())
                && row.content() == &CardRowContent::Address(name.clone())
        }));

        assert_eq!(window.cards()[1].address(), &CardAddress::Value(age));
        assert_eq!(window.cards()[2].address(), &CardAddress::Value(name));
        assert_eq!(window.cards()[1].owned_slot(), None);
        assert_eq!(window.cards()[2].owned_slot(), None);
        assert!(window.cards()[2].rows().iter().any(|row| {
            row.key() == &CardRowKey::Value
                && row.content() == &CardRowContent::Text("Ada".to_owned())
        }));
        assert_eq!(
            arena.allocated_slot_count(),
            1,
            "cards and rows are projections, not ownership identities"
        );
    }

    #[test]
    fn card_and_relationship_budgets_bound_large_sequence_observation() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime((0_usize..1_000_000).collect::<Vec<_>>()))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);

        let window = CardWindow::first(&source, NonZeroUsize::new(8).unwrap(), 3).unwrap();

        assert_eq!(window.cards().len(), 8);
        assert!(window.has_after());
        assert_eq!(window.cards()[0].rows().len(), 4);
        assert!(!window.cards()[0].relationships_complete());
        assert!(
            window
                .cards()
                .iter()
                .skip(1)
                .all(|card| card.owned_slot().is_none())
        );
    }
}
