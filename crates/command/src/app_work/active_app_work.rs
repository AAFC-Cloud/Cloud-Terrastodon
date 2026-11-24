use std::time::Instant;
use tokio::task::Id;

/// A display snapshot of an admitted background job.
#[derive(Debug, Clone)]
pub struct ActiveAppWork {
    pub id: Id,
    pub description: String,
    pub started_at: Instant,
}
