use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PickerEvent {
    InitialLoad,
    QueryChanged(Arc<str>),
    QueryCleared,
    ReloadRequested,
}
