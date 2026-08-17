use super::card_address::CardAddress;
use super::revision::ScanRevisionStamp;
use super::value_address::ValueAddress;
use super::work_budget::WorkBudget;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ScanProgress {
    pub(crate) inspected: usize,
    pub(crate) matched: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum QueryTotal {
    #[default]
    Unknown,
    Scanning(ScanProgress),
    Exact(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ScanCandidate {
    Match(ValueAddress),
    NoMatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EndScanPoll {
    Pending(ScanProgress),
    Complete(ScanProgress),
    Cancelled,
    Stale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EndScanState {
    Running,
    Complete,
    Cancelled,
    Stale,
}

/// Cooperative complete-stream scan used when a query cannot seek directly to
/// its final match.
///
/// The previously published selection is intentionally stored separately from
/// `last_match`: no partial scan result becomes visible.
pub(crate) struct EndScan<I>
where
    I: Iterator<Item = ScanCandidate>,
{
    candidates: I,
    stamp: ScanRevisionStamp,
    progress: ScanProgress,
    last_match: Option<ValueAddress>,
    selection: CardAddress,
    total: QueryTotal,
    state: EndScanState,
}

impl<I> EndScan<I>
where
    I: Iterator<Item = ScanCandidate>,
{
    pub(crate) fn new(candidates: I, stamp: ScanRevisionStamp, selection: CardAddress) -> Self {
        Self {
            candidates,
            stamp,
            progress: ScanProgress::default(),
            last_match: None,
            selection,
            total: QueryTotal::Scanning(ScanProgress::default()),
            state: EndScanState::Running,
        }
    }

    pub(crate) fn selection(&self) -> &CardAddress {
        &self.selection
    }

    pub(crate) const fn total(&self) -> QueryTotal {
        self.total
    }

    pub(crate) fn cancel(&mut self) {
        if self.state == EndScanState::Running {
            self.state = EndScanState::Cancelled;
            self.total = QueryTotal::Unknown;
        }
    }

    pub(crate) fn poll(
        &mut self,
        current_stamp: ScanRevisionStamp,
        budget: &mut WorkBudget,
    ) -> EndScanPoll {
        match self.state {
            EndScanState::Complete => return EndScanPoll::Complete(self.progress),
            EndScanState::Cancelled => return EndScanPoll::Cancelled,
            EndScanState::Stale => return EndScanPoll::Stale,
            EndScanState::Running => {}
        }

        if current_stamp != self.stamp {
            self.state = EndScanState::Stale;
            self.total = QueryTotal::Unknown;
            return EndScanPoll::Stale;
        }

        while budget.try_consume() {
            match self.candidates.next() {
                Some(ScanCandidate::Match(address)) => {
                    self.progress.inspected += 1;
                    self.progress.matched += 1;
                    self.last_match = Some(address);
                }
                Some(ScanCandidate::NoMatch) => {
                    self.progress.inspected += 1;
                }
                None => {
                    self.state = EndScanState::Complete;
                    self.selection = self
                        .last_match
                        .clone()
                        .map(CardAddress::Value)
                        .unwrap_or(CardAddress::NewSlot);
                    self.total = QueryTotal::Exact(self.progress.matched);
                    return EndScanPoll::Complete(self.progress);
                }
            }
        }

        self.total = QueryTotal::Scanning(self.progress);
        EndScanPoll::Pending(self.progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::revision::ArenaRevisions;
    use crate::object_explorer::revision::QueryRevision;
    use crate::object_explorer::slot_id::SlotId;
    use crate::object_explorer::value_address::ValueAddress;

    fn stamp() -> ScanRevisionStamp {
        ArenaRevisions::default().scan_stamp(QueryRevision::default())
    }

    fn selected(slot: u64) -> CardAddress {
        CardAddress::Value(ValueAddress::root(SlotId::new(slot)))
    }

    #[test]
    fn ordinary_navigation_does_not_request_exact_total() {
        assert_eq!(QueryTotal::default(), QueryTotal::Unknown);
    }

    #[test]
    fn end_scan_is_budgeted_cancellable_and_atomic() {
        let original_selection = selected(900);
        let candidates = (0_u64..100).map(|index| {
            if index % 10 == 0 {
                ScanCandidate::Match(ValueAddress::root(SlotId::new(index)))
            } else {
                ScanCandidate::NoMatch
            }
        });
        let mut scan = EndScan::new(candidates, stamp(), original_selection.clone());
        let mut first_budget = WorkBudget::new(7);

        assert_eq!(
            scan.poll(stamp(), &mut first_budget),
            EndScanPoll::Pending(ScanProgress {
                inspected: 7,
                matched: 1,
            })
        );
        assert_eq!(first_budget.spent(), 7);
        assert_eq!(scan.selection(), &original_selection);
        assert_eq!(
            scan.total(),
            QueryTotal::Scanning(ScanProgress {
                inspected: 7,
                matched: 1,
            })
        );

        scan.cancel();
        let mut cancelled_budget = WorkBudget::new(7);
        assert_eq!(
            scan.poll(stamp(), &mut cancelled_budget),
            EndScanPoll::Cancelled
        );
        assert_eq!(scan.selection(), &original_selection);
        assert_eq!(scan.total(), QueryTotal::Unknown);
        assert_eq!(cancelled_budget.spent(), 0);
    }

    #[test]
    fn exact_total_is_published_only_after_exhaustion() {
        let candidates = [
            ScanCandidate::Match(ValueAddress::root(SlotId::new(2))),
            ScanCandidate::NoMatch,
            ScanCandidate::Match(ValueAddress::root(SlotId::new(8))),
        ];
        let mut scan = EndScan::new(candidates.into_iter(), stamp(), selected(100));
        let mut exact_item_budget = WorkBudget::new(3);

        assert!(matches!(
            scan.poll(stamp(), &mut exact_item_budget),
            EndScanPoll::Pending(_)
        ));
        assert_eq!(scan.selection(), &selected(100));
        assert!(matches!(scan.total(), QueryTotal::Scanning(_)));

        let mut exhaustion_budget = WorkBudget::new(1);
        assert_eq!(
            scan.poll(stamp(), &mut exhaustion_budget),
            EndScanPoll::Complete(ScanProgress {
                inspected: 3,
                matched: 2,
            })
        );
        assert_eq!(scan.selection(), &selected(8));
        assert_eq!(scan.total(), QueryTotal::Exact(2));
    }

    #[test]
    fn end_scan_stales_on_arena_or_query_revision_change() {
        let original_selection = selected(40);
        let candidates = std::iter::repeat_n(ScanCandidate::NoMatch, 100);
        let initial_stamp = stamp();
        let mut scan = EndScan::new(candidates, initial_stamp, original_selection.clone());
        let stale_stamp = ScanRevisionStamp {
            arena: initial_stamp.arena,
            query: initial_stamp.query.next(),
        };
        let mut budget = WorkBudget::new(10);

        assert_eq!(scan.poll(stale_stamp, &mut budget), EndScanPoll::Stale);
        assert_eq!(budget.spent(), 0);
        assert_eq!(scan.selection(), &original_selection);
        assert_eq!(scan.total(), QueryTotal::Unknown);
    }

    #[test]
    fn million_candidate_no_match_scan_obeys_each_budget() {
        let candidates = std::iter::repeat_n(ScanCandidate::NoMatch, 1_000_000);
        let mut scan = EndScan::new(candidates, stamp(), selected(1));
        let mut budget = WorkBudget::new(8);

        assert_eq!(
            scan.poll(stamp(), &mut budget),
            EndScanPoll::Pending(ScanProgress {
                inspected: 8,
                matched: 0,
            })
        );
        assert_eq!(budget.spent(), 8);
        assert_eq!(scan.selection(), &selected(1));
        assert_eq!(
            std::mem::size_of_val(&scan),
            std::mem::size_of::<EndScan<std::iter::RepeatN<ScanCandidate>>>(),
            "scan state is independent of candidate count"
        );
    }
}
