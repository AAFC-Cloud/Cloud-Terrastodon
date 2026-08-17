/// Stable semantic identity for a row within one card.
///
/// Presentation order and terminal line numbers are deliberately absent.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CardRowKey {
    Shape,
    Variant,
    Value,
    Field(String),
    Element(usize),
    MapValue(String),
    Status,
    Action(String),
}
