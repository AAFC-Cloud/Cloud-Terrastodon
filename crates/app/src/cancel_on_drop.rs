use teamy_cancellation::CancellationToken;

pub(crate) struct CancelOnDrop(pub(crate) CancellationToken);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if !self.0.is_cancelled() {
            // Routine shutdown is not a user cancellation event. Signal clones
            // without adding the token library's INFO message to every command.
            tracing::subscriber::with_default(tracing::subscriber::NoSubscriber::default(), || {
                self.0.request_cancel("application invocation finished");
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_guard_requests_cooperative_cancellation() {
        let token = CancellationToken::new();
        let guard = CancelOnDrop(token.clone());
        assert!(!token.is_cancelled());
        drop(guard);
        assert!(token.is_cancelled());
    }
}
