use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BrowseSessionId(u64);

impl BrowseSessionId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

/// Independent cooperative and presentation bounds for one UI frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CardWindowBudget {
    max_work: usize,
    max_cards: NonZeroUsize,
    max_relationship_rows: usize,
}

impl CardWindowBudget {
    pub(crate) const fn new(
        max_work: usize,
        max_cards: NonZeroUsize,
        max_relationship_rows: usize,
    ) -> Self {
        Self {
            max_work,
            max_cards,
            max_relationship_rows,
        }
    }

    pub(crate) const fn max_work(self) -> usize {
        self.max_work
    }

    pub(crate) const fn max_cards(self) -> NonZeroUsize {
        self.max_cards
    }

    pub(crate) const fn max_relationship_rows(self) -> usize {
        self.max_relationship_rows
    }
}
