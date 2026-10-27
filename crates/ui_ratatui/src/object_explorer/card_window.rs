use super::arena_address_source::ArenaAddressSource;
use super::card_snapshot::CardSnapshot;
use super::preorder_cursor::PreorderCursor;
use super::query_window::QueryWindow;
use super::value_resolution_error::ValueResolutionError;
use std::num::NonZeroUsize;

/// A bounded set of card snapshots for one presentation request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CardWindow {
    cards: Vec<CardSnapshot>,
    start_ordinal: usize,
    has_before: bool,
    has_after: bool,
}

impl CardWindow {
    pub(crate) fn from_cards(
        cards: Vec<CardSnapshot>,
        start_ordinal: usize,
        has_before: bool,
        has_after: bool,
    ) -> Self {
        Self {
            cards,
            start_ordinal,
            has_before,
            has_after,
        }
    }

    pub(crate) fn single(card: CardSnapshot) -> Self {
        Self {
            cards: vec![card],
            start_ordinal: 0,
            has_before: false,
            has_after: false,
        }
    }

    pub(crate) fn first(
        source: &ArenaAddressSource<'_>,
        max_cards: NonZeroUsize,
        max_relationship_rows: usize,
    ) -> Result<Self, ValueResolutionError> {
        let mut cursor = PreorderCursor::new(source);
        let mut cards = Vec::with_capacity(max_cards.get());
        for _ in 0..max_cards.get() {
            let Some(address) = cursor.next() else {
                break;
            };
            cards.push(CardSnapshot::observe(
                source,
                address,
                max_relationship_rows,
            )?);
        }
        let has_after = cursor.next().is_some();
        Ok(Self {
            cards,
            start_ordinal: 0,
            has_before: false,
            has_after,
        })
    }

    pub(crate) fn cards(&self) -> &[CardSnapshot] {
        &self.cards
    }

    pub(crate) const fn start_ordinal(&self) -> usize {
        self.start_ordinal
    }

    /// Append the logical create-object card to this bounded snapshot window.
    /// The renderer clips the extra card to the viewport, preserving real cards
    /// until navigation actually needs to reveal the create-object card.
    pub(crate) fn including_new_slot(&self) -> Self {
        let mut cards = self.cards.clone();
        cards.push(CardSnapshot::new_slot());
        Self {
            cards,
            start_ordinal: self.start_ordinal,
            has_before: self.has_before,
            has_after: false,
        }
    }

    pub(crate) fn observe_query_window(
        source: &ArenaAddressSource<'_>,
        window: QueryWindow,
        max_relationship_rows: usize,
    ) -> Result<Self, ValueResolutionError> {
        let cards = window
            .addresses()
            .iter()
            .cloned()
            .map(|address| CardSnapshot::observe(source, address, max_relationship_rows))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            cards,
            start_ordinal: window.start_ordinal(),
            has_before: window.has_before(),
            has_after: window.has_after(),
        })
    }

    pub(crate) const fn has_before(&self) -> bool {
        self.has_before
    }

    pub(crate) const fn has_after(&self) -> bool {
        self.has_after
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::arena::Arena;
    use crate::object_explorer::card_address::CardAddress;
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    #[test]
    fn new_slot_appends_without_discarding_tail_snapshots() {
        let mut arena = Arena::default();
        for value in 0_usize..5 {
            arena.insert_ready(runtime(value)).unwrap();
        }
        let source = ArenaAddressSource::new(&arena);
        let window = CardWindow::first(&source, NonZeroUsize::new(5).unwrap(), 1).unwrap();
        assert!(!window.has_after());

        let with_new = window.including_new_slot();

        assert_eq!(with_new.cards().len(), 6);
        assert_eq!(&with_new.cards()[..5], window.cards());
        assert_eq!(with_new.start_ordinal(), window.start_ordinal());
        assert_eq!(
            with_new.cards().last().map(CardSnapshot::address),
            Some(&CardAddress::NewSlot)
        );
        assert!(!with_new.has_before());
        assert!(!with_new.has_after());
    }
}
