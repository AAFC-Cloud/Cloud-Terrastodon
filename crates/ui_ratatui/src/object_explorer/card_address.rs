use super::value_address::ValueAddress;

/// Logical identity of a navigable card.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CardAddress {
    Value(ValueAddress),
    NewSlot,
}
