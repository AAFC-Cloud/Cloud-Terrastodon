pub(super) enum RunOutcome<T> {
    Selected(Vec<T>),
    Cancelled,
    ReloadRequested,
    NoChoices,
}
