use std::time::Duration;

/// Progress reported by the initializer while assembling a paginated collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectionProgress {
    pub total: Option<usize>,
    pub accumulated: usize,
    pub received_this_page: usize,
    pub time_remaining: Duration,
}
