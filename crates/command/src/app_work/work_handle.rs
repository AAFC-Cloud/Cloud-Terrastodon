use teamy_cancellation::CancellationToken;

/// Cancellation and diagnostics for a task owned by the tracker's task set.
#[derive(Debug)]
pub(super) struct WorkHandle {
    pub description: String,
    pub cancellation: CancellationToken,
}
