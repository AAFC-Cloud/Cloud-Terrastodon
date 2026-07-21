#[derive(Debug, Eq, PartialEq)]
pub(super) enum QueryEvent {
    Changed(String),
    Cleared,
}
