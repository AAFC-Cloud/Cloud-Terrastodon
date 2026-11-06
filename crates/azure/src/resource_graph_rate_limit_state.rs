use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResourceGraphRateLimitState {
    pub(crate) blocked_until: Option<Instant>,
}

impl ResourceGraphRateLimitState {
    pub(crate) const fn new() -> Self {
        Self {
            blocked_until: None,
        }
    }
}
