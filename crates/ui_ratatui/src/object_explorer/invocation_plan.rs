use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::Thing;

use super::invocation_host::InvocationId;
use super::invocation_mode::InvocationMode;
use super::slot_id::SlotId;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct InvocationPlanId(u64);

impl InvocationPlanId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PlannedInvocation {
    pub(crate) input: SlotId,
    pub(crate) input_thing: &'static Thing,
    pub(crate) function: &'static Function,
    pub(crate) mode: InvocationMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InvocationPlanStepStatus {
    Planned,
    Waiting {
        invocation: InvocationId,
        output: SlotId,
    },
    Complete {
        output: SlotId,
    },
    Failed(String),
    Cancelled,
}

pub(crate) struct InvocationPlanStep {
    operation: PlannedInvocation,
    status: InvocationPlanStepStatus,
}

impl InvocationPlanStep {
    pub(crate) const fn operation(&self) -> PlannedInvocation {
        self.operation
    }

    pub(crate) const fn status(&self) -> &InvocationPlanStepStatus {
        &self.status
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InvocationPlanCompletion {
    Planned,
    Waiting,
    Complete,
    Failed,
    Cancelled,
}

/// Engine metadata for sequential reflected invocations.
///
/// A plan has its own identity and never receives an Arena SlotId. Only actual
/// invocation inputs and outputs are arena roots.
pub(crate) struct InvocationPlan {
    id: InvocationPlanId,
    title: String,
    steps: Vec<InvocationPlanStep>,
    cancelled: bool,
}

impl InvocationPlan {
    pub(crate) fn new(
        id: InvocationPlanId,
        title: impl Into<String>,
        operations: impl IntoIterator<Item = PlannedInvocation>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            steps: operations
                .into_iter()
                .map(|operation| InvocationPlanStep {
                    operation,
                    status: InvocationPlanStepStatus::Planned,
                })
                .collect(),
            cancelled: false,
        }
    }

    pub(crate) const fn id(&self) -> InvocationPlanId {
        self.id
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn steps(&self) -> &[InvocationPlanStep] {
        &self.steps
    }

    pub(crate) fn completion(&self) -> InvocationPlanCompletion {
        if self.cancelled {
            return InvocationPlanCompletion::Cancelled;
        }
        if self
            .steps
            .iter()
            .any(|step| matches!(step.status, InvocationPlanStepStatus::Failed(_)))
        {
            return InvocationPlanCompletion::Failed;
        }
        if self
            .steps
            .iter()
            .all(|step| matches!(step.status, InvocationPlanStepStatus::Complete { .. }))
        {
            return InvocationPlanCompletion::Complete;
        }
        if self
            .steps
            .iter()
            .any(|step| matches!(step.status, InvocationPlanStepStatus::Waiting { .. }))
        {
            InvocationPlanCompletion::Waiting
        } else {
            InvocationPlanCompletion::Planned
        }
    }

    pub(crate) fn next_planned(&self) -> Option<(usize, PlannedInvocation)> {
        if self.cancelled
            || self.steps.iter().any(|step| {
                matches!(
                    step.status,
                    InvocationPlanStepStatus::Waiting { .. }
                        | InvocationPlanStepStatus::Failed(_)
                        | InvocationPlanStepStatus::Cancelled
                )
            })
        {
            return None;
        }
        self.steps.iter().enumerate().find_map(|(index, step)| {
            matches!(step.status, InvocationPlanStepStatus::Planned)
                .then_some((index, step.operation))
        })
    }

    pub(crate) fn mark_waiting(&mut self, step: usize, invocation: InvocationId, output: SlotId) {
        self.steps[step].status = InvocationPlanStepStatus::Waiting { invocation, output };
    }

    pub(crate) fn mark_complete(&mut self, step: usize, output: SlotId) {
        self.steps[step].status = InvocationPlanStepStatus::Complete { output };
    }

    pub(crate) fn mark_failed(&mut self, step: usize, message: impl Into<String>) {
        self.steps[step].status = InvocationPlanStepStatus::Failed(message.into());
    }

    pub(crate) fn mark_cancelled(&mut self, step: usize) {
        self.steps[step].status = InvocationPlanStepStatus::Cancelled;
    }

    pub(crate) fn cancel(&mut self) {
        self.cancelled = true;
        for step in &mut self.steps {
            if !matches!(step.status, InvocationPlanStepStatus::Complete { .. }) {
                step.status = InvocationPlanStepStatus::Cancelled;
            }
        }
    }
}
