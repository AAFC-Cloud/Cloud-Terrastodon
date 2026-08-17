use super::slot_id::SlotId;
use super::value_builder::BuilderTransition;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ProductionJobId(u64);

impl ProductionJobId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ProductionJobId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProductionStrategy {
    Default,
    Manual,
    Arbitrary { bytes: Vec<u8> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProductionJobState {
    Running {
        latest_root: Option<SlotId>,
    },
    Complete {
        output: SlotId,
        destination_transition: BuilderTransition,
    },
    Failed {
        message: String,
    },
}

impl ProductionJobState {
    pub(crate) const fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete { .. } | Self::Failed { .. })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProductionUpdate {
    job: ProductionJobId,
    destination: SlotId,
    field: usize,
    input: Option<SlotId>,
    state: ProductionJobState,
}

impl ProductionUpdate {
    pub(super) const fn new(
        job: ProductionJobId,
        destination: SlotId,
        field: usize,
        input: Option<SlotId>,
        state: ProductionJobState,
    ) -> Self {
        Self {
            job,
            destination,
            field,
            input,
            state,
        }
    }

    pub(crate) const fn job(&self) -> ProductionJobId {
        self.job
    }

    pub(crate) const fn destination(&self) -> SlotId {
        self.destination
    }

    pub(crate) const fn field(&self) -> usize {
        self.field
    }

    /// The outer producer function's request root, once one exists.
    pub(crate) const fn input(&self) -> Option<SlotId> {
        self.input
    }

    pub(crate) const fn state(&self) -> &ProductionJobState {
        &self.state
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProductionBatch {
    updates: Vec<ProductionUpdate>,
    active_jobs: usize,
    work_spent: usize,
}

impl ProductionBatch {
    pub(super) const fn new(
        updates: Vec<ProductionUpdate>,
        active_jobs: usize,
        work_spent: usize,
    ) -> Self {
        Self {
            updates,
            active_jobs,
            work_spent,
        }
    }

    pub(crate) fn updates(&self) -> &[ProductionUpdate] {
        &self.updates
    }

    pub(crate) const fn active_jobs(&self) -> usize {
        self.active_jobs
    }

    pub(crate) const fn work_spent(&self) -> usize {
        self.work_spent
    }
}
