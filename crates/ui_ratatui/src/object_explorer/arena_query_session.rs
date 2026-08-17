use super::work_budget::WorkBudget;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct QuerySessionId(u64);

impl QuerySessionId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct JsonBatch {
    /// Already serialized bounded fragment. It never contains a RuntimeValue or
    /// Facet Peek whose lifetime could cross the command channel.
    pub(crate) fragment: String,
    pub(crate) inspected: usize,
    pub(crate) emitted: usize,
    pub(crate) complete: bool,
}

/// Independent bounds for one engine-to-writer transfer.
///
/// `work` limits reflected cursor advancement. `max_bytes` limits the
/// serialized fragment retained by the engine and sent through the command
/// channel. A single reflected value larger than `max_bytes` is rejected
/// rather than silently defeating backpressure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JsonBatchBudget {
    work: WorkBudget,
    max_bytes: usize,
}

impl JsonBatchBudget {
    pub(crate) const fn new(max_work: usize, max_bytes: usize) -> Self {
        Self {
            work: WorkBudget::new(max_work),
            max_bytes,
        }
    }

    pub(crate) const fn max_work(self) -> usize {
        self.work.limit()
    }

    pub(crate) const fn max_bytes(self) -> usize {
        self.max_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QuerySessionEnd {
    Complete,
    Cancelled,
}
