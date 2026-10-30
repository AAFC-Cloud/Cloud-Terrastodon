use chrono::DateTime;
use chrono::Utc;
use std::time::Instant;

/// Correlated wall and monotonic time when an execution outcome was observed.
/// Capture this before decoding or staging; those operations do not renew age.
#[derive(Clone, Copy, Debug)]
pub struct ArtifactObservation {
    // The clocks belong to one observation and cannot be replaced independently.
    pub(crate) wall: DateTime<Utc>,
    pub(crate) monotonic: Instant,
}

impl ArtifactObservation {
    pub fn now() -> Self {
        Self {
            monotonic: Instant::now(),
            wall: Utc::now(),
        }
    }
}
