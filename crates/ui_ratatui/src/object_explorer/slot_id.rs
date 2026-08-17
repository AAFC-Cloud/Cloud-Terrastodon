use std::fmt;

/// Stable identity of one ownership-bearing root in an explorer arena.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct SlotId(u64);

impl SlotId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for SlotId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::SlotId;

    #[test]
    fn slot_id_orders_roots_by_monotonic_identity() {
        let mut ids = [SlotId::new(9), SlotId::new(2), SlotId::new(5)];
        ids.sort();

        assert_eq!(
            ids.map(SlotId::get),
            [2, 5, 9],
            "root traversal can use SlotId order without a flattened card ordinal"
        );
    }
}
