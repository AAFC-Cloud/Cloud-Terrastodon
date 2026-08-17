use super::arena_address_source::ArenaAddressSource;
use super::card_address::CardAddress;

/// Durable logical card selection.
///
/// No flattened ordinal is stored. A presentation layer may calculate a
/// temporary window position for rendering, but insertion/removal of cards
/// before this address cannot silently retarget the selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CardSelection {
    selected: CardAddress,
}

impl CardSelection {
    pub(crate) const fn new(selected: CardAddress) -> Self {
        Self { selected }
    }

    pub(crate) const fn selected(&self) -> &CardAddress {
        &self.selected
    }

    pub(crate) fn select(&mut self, selected: CardAddress) {
        self.selected = selected;
    }

    /// Preserve a still-resolving logical address exactly.
    ///
    /// Full nearest-successor/predecessor re-anchoring belongs to
    /// CardNavigator. This conservative fallback is sufficient when the
    /// selected value itself disappeared and never reacts to unrelated roots.
    pub(crate) fn reconcile(&mut self, source: &ArenaAddressSource<'_>) {
        let CardAddress::Value(address) = &self.selected else {
            return;
        };
        if source.resolve(address).is_err() {
            self.selected = CardAddress::NewSlot;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::arena::Arena;
    use crate::object_explorer::preorder_cursor::PreorderCursor;
    use crate::object_explorer::value_address::ValueAddress;
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct EarlierProjection {
        values: Vec<String>,
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    #[test]
    fn selection_remains_on_owned_slot_when_prior_projection_set_grows() {
        let mut arena = Arena::default();
        let earlier_pending = arena.insert_pending().unwrap();
        arena
            .insert_ready(runtime(String::from("middle one")))
            .unwrap();
        arena
            .insert_ready(runtime(String::from("middle two")))
            .unwrap();
        let selected_slot = arena
            .insert_ready(runtime(String::from("selected")))
            .unwrap();
        let selected_address = ValueAddress::root(selected_slot);
        let mut selection = CardSelection::new(CardAddress::Value(selected_address.clone()));

        let before_source = ArenaAddressSource::new(&arena);
        let before = PreorderCursor::new(&before_source).collect::<Vec<_>>();
        let before_ordinal = before
            .iter()
            .position(|address| address == &selected_address)
            .expect("selected root is visible before producer completion");

        arena
            .set_ready(
                earlier_pending,
                runtime(EarlierProjection {
                    values: (0..12).map(|index| format!("new {index}")).collect(),
                }),
            )
            .unwrap();

        let after_source = ArenaAddressSource::new(&arena);
        selection.reconcile(&after_source);
        let after = PreorderCursor::new(&after_source).collect::<Vec<_>>();
        let after_ordinal = after
            .iter()
            .position(|address| address == &selected_address)
            .expect("selected root remains visible after producer completion");

        assert!(after_ordinal > before_ordinal);
        assert_eq!(
            selection.selected(),
            &CardAddress::Value(selected_address),
            "selection identity must not be today's flattened card ordinal"
        );
    }
}
