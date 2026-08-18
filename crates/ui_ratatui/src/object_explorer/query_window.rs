use super::value_address::ValueAddress;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueryWindow {
    addresses: Vec<ValueAddress>,
    start_ordinal: usize,
    has_before: bool,
    has_after: bool,
}

impl QueryWindow {
    pub(crate) fn new(
        addresses: Vec<ValueAddress>,
        start_ordinal: usize,
        has_before: bool,
        has_after: bool,
    ) -> Self {
        Self {
            addresses,
            start_ordinal,
            has_before,
            has_after,
        }
    }

    pub(crate) fn addresses(&self) -> &[ValueAddress] {
        &self.addresses
    }

    pub(crate) const fn start_ordinal(&self) -> usize {
        self.start_ordinal
    }

    pub(crate) const fn has_before(&self) -> bool {
        self.has_before
    }

    pub(crate) const fn has_after(&self) -> bool {
        self.has_after
    }
}
