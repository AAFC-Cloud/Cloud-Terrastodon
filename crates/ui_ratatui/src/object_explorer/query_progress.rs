use super::end_scan::QueryTotal;
use super::query_instrumentation::QueryInstrumentation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum QueryProgressState<T> {
    Ready(T),
    Pending,
    Complete,
    Cancelled,
    Stale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueryProgress<T> {
    state: QueryProgressState<T>,
    work_spent: usize,
    instrumentation: QueryInstrumentation,
    total: QueryTotal,
}

impl<T> QueryProgress<T> {
    pub(crate) const fn new(
        state: QueryProgressState<T>,
        work_spent: usize,
        instrumentation: QueryInstrumentation,
        total: QueryTotal,
    ) -> Self {
        Self {
            state,
            work_spent,
            instrumentation,
            total,
        }
    }

    pub(crate) const fn state(&self) -> &QueryProgressState<T> {
        &self.state
    }

    pub(crate) const fn work_spent(&self) -> usize {
        self.work_spent
    }

    pub(crate) const fn instrumentation(&self) -> QueryInstrumentation {
        self.instrumentation
    }

    pub(crate) const fn total(&self) -> QueryTotal {
        self.total
    }

    pub(crate) fn into_state(self) -> QueryProgressState<T> {
        self.state
    }
}
