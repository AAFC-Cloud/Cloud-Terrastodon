use super::query_event::QueryEvent;
use std::time::Duration;
use std::time::Instant;

pub(super) const QUERY_DEBOUNCE: Duration = Duration::from_millis(50);

#[derive(Debug, Default)]
pub(super) struct QueryDebouncer {
    pending: Option<String>,
    deadline: Option<Instant>,
}

impl QueryDebouncer {
    pub(super) fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub(super) fn schedule(&mut self, query: String, now: Instant) {
        self.pending = Some(query);
        self.deadline = Some(now + QUERY_DEBOUNCE);
    }

    pub(super) fn clear(&mut self) {
        self.pending = None;
        self.deadline = None;
    }

    pub(super) fn take_due(&mut self, now: Instant) -> Option<QueryEvent> {
        if !self.deadline.is_some_and(|deadline| now >= deadline) {
            return None;
        }
        self.deadline = None;
        match self.pending.take()? {
            query if query.is_empty() => Some(QueryEvent::Cleared),
            query => Some(QueryEvent::Changed(query)),
        }
    }
}
