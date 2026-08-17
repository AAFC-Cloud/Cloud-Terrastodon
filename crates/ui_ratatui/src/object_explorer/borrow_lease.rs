use super::slot_id::SlotId;
use super::value_address::ValueAddress;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct BorrowId(u64);

impl BorrowId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BorrowHolder {
    Builder(SlotId),
    ReadyValue(SlotId),
    PendingInvocation(SlotId),
}

/// Unique engine-owned token retaining one reflected borrow relationship.
///
/// This token is intentionally not Clone. Moving it between builder, ready
/// value, and pending-invocation state transfers responsibility for releasing
/// the edge.
#[derive(Debug)]
pub(crate) struct BorrowLease {
    id: BorrowId,
    source: ValueAddress,
    holder: BorrowHolder,
    field: String,
}

impl BorrowLease {
    pub(crate) fn new(
        id: BorrowId,
        source: ValueAddress,
        holder: BorrowHolder,
        field: String,
    ) -> Self {
        Self {
            id,
            source,
            holder,
            field,
        }
    }

    pub(crate) const fn id(&self) -> BorrowId {
        self.id
    }

    pub(crate) fn source(&self) -> &ValueAddress {
        &self.source
    }

    pub(crate) const fn holder(&self) -> BorrowHolder {
        self.holder
    }

    pub(crate) fn field(&self) -> &str {
        &self.field
    }

    pub(crate) fn transfer(&mut self, holder: BorrowHolder) {
        self.holder = holder;
    }
}
