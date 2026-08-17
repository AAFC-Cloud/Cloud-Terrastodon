use super::value_candidate::ValueCandidate;
use std::num::NonZeroUsize;

/// Bounded picker choices retaining only addresses and generic provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValueCandidateWindow {
    candidates: Vec<ValueCandidate>,
    has_before: bool,
    has_after: bool,
}

impl ValueCandidateWindow {
    pub(crate) fn new(candidates: Vec<ValueCandidate>, has_before: bool, has_after: bool) -> Self {
        Self {
            candidates,
            has_before,
            has_after,
        }
    }

    pub(crate) fn candidates(&self) -> &[ValueCandidate] {
        &self.candidates
    }

    pub(crate) const fn has_before(&self) -> bool {
        self.has_before
    }

    pub(crate) const fn has_after(&self) -> bool {
        self.has_after
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ValueCandidateWindowBudget {
    max_work: usize,
    max_candidates: NonZeroUsize,
}

impl ValueCandidateWindowBudget {
    pub(crate) const fn new(max_work: usize, max_candidates: NonZeroUsize) -> Self {
        Self {
            max_work,
            max_candidates,
        }
    }

    pub(crate) const fn max_work(self) -> usize {
        self.max_work
    }

    pub(crate) const fn max_candidates(self) -> NonZeroUsize {
        self.max_candidates
    }
}
