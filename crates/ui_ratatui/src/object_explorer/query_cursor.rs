use super::arena_address_source::ArenaAddressSource;
use super::card_address::CardAddress;
use super::card_navigation::CardNavigation;
use super::end_scan::QueryTotal;
use super::end_scan::ScanProgress;
use super::query_instrumentation::QueryInstrumentation;
use super::query_plan::QueryPlan;
use super::query_plan::QueryPlanInstrumentation;
use super::query_plan::QueryPlanIter;
use super::query_plan::QueryPlanPoll;
use super::query_progress::QueryProgress;
use super::query_progress::QueryProgressState;
use super::query_window::QueryWindow;
use super::revision::ScanRevisionStamp;
use super::value_address::ValueAddress;
use super::work_budget::WorkBudget;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QueryCursorError {
    WindowExceedsCache {
        requested: usize,
        required: usize,
        capacity: usize,
    },
    OrdinalOverflow,
    MissingWindow {
        start: usize,
        max_cards: usize,
    },
}

impl fmt::Display for QueryCursorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WindowExceedsCache {
                requested,
                required,
                capacity,
            } => write!(
                formatter,
                "a {requested}-card window needs {required} cache entries including lookahead, but capacity is {capacity}"
            ),
            Self::OrdinalOverflow => write!(formatter, "query result ordinal space exhausted"),
            Self::MissingWindow { start, max_cards } => write!(
                formatter,
                "query cache cannot provide {max_cards} cards starting at ordinal {start}"
            ),
        }
    }
}

impl Error for QueryCursorError {}

#[derive(Debug)]
enum ActiveOperation {
    Seek(ValueAddress),
    Fill {
        anchor: Option<ValueAddress>,
        max_cards: usize,
        start_ordinal: Option<usize>,
    },
    End {
        last_match: Option<ValueAddress>,
        matched: usize,
    },
}

struct AddressCache {
    capacity: usize,
    entries: BTreeMap<usize, ValueAddress>,
    insertion_order: VecDeque<usize>,
}

impl AddressCache {
    fn new(capacity: NonZeroUsize) -> Self {
        Self {
            capacity: capacity.get(),
            entries: BTreeMap::new(),
            insertion_order: VecDeque::new(),
        }
    }

    fn insert(&mut self, ordinal: usize, address: ValueAddress) {
        self.insertion_order
            .retain(|candidate| *candidate != ordinal);
        self.entries.insert(ordinal, address);
        self.insertion_order.push_back(ordinal);
        while self.entries.len() > self.capacity {
            let Some(evicted) = self.insertion_order.pop_front() else {
                break;
            };
            self.entries.remove(&evicted);
        }
    }

    fn get(&self, ordinal: usize) -> Option<&ValueAddress> {
        self.entries.get(&ordinal)
    }

    fn ordinal_of(&self, address: &ValueAddress) -> Option<usize> {
        self.entries
            .iter()
            .find_map(|(ordinal, candidate)| (candidate == address).then_some(*ordinal))
    }

    fn contains_range(&self, start: usize, end_inclusive: usize) -> bool {
        (start..=end_inclusive).all(|ordinal| self.entries.contains_key(&ordinal))
    }

    fn collect_range(&self, start: usize, end_exclusive: usize) -> Option<Vec<ValueAddress>> {
        (start..end_exclusive)
            .map(|ordinal| self.entries.get(&ordinal).cloned())
            .collect()
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.insertion_order.clear();
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

enum ScannerPoll {
    Item {
        ordinal: usize,
        address: ValueAddress,
    },
    Pending,
    Complete,
}

/// Revision-scoped, bounded-memory navigation over a lazy QueryPlan.
///
/// `selection` is always a logical address. Result ordinals are transient
/// accelerators valid only for `stamp`; they are never exposed as identity.
pub(crate) struct QueryCursor<'source, 'arena> {
    source: &'source ArenaAddressSource<'arena>,
    plan: QueryPlan,
    stamp: ScanRevisionStamp,
    scanner: QueryPlanIter<'source, 'arena>,
    total_scanner: QueryPlanIter<'source, 'arena>,
    total_matched: usize,
    scanner_next_ordinal: usize,
    scanner_complete: bool,
    scanner_error: Option<QueryCursorError>,
    retired: QueryPlanInstrumentation,
    cache: AddressCache,
    selection: CardAddress,
    selection_ordinal: Option<usize>,
    active: Option<ActiveOperation>,
    total: QueryTotal,
    serialized: usize,
    stale: bool,
}

impl<'source, 'arena> QueryCursor<'source, 'arena>
where
    'arena: 'source,
{
    pub(crate) fn new(
        source: &'source ArenaAddressSource<'arena>,
        plan: QueryPlan,
        stamp: ScanRevisionStamp,
        cache_capacity: NonZeroUsize,
    ) -> Self {
        let scanner = plan.evaluate(source);
        let total_scanner = plan.evaluate(source);
        Self {
            source,
            plan,
            stamp,
            scanner,
            total_scanner,
            total_matched: 0,
            scanner_next_ordinal: 0,
            scanner_complete: false,
            scanner_error: None,
            retired: QueryPlanInstrumentation::default(),
            cache: AddressCache::new(cache_capacity),
            selection: CardAddress::NewSlot,
            selection_ordinal: None,
            active: None,
            total: QueryTotal::Unknown,
            serialized: 0,
            stale: false,
        }
    }

    pub(crate) const fn selection(&self) -> &CardAddress {
        &self.selection
    }

    pub(crate) const fn total(&self) -> QueryTotal {
        self.total
    }

    pub(crate) fn instrumentation(&self) -> QueryInstrumentation {
        QueryInstrumentation::from_plan(
            self.retired,
            self.scanner.instrumentation(),
            self.serialized,
            self.cache.len(),
        )
    }

    pub(crate) fn record_serialized(&mut self, count: usize) {
        self.serialized = self.serialized.saturating_add(count);
    }

    pub(crate) fn next(
        &mut self,
        current_stamp: ScanRevisionStamp,
        budget: &mut WorkBudget,
    ) -> Result<QueryProgress<ValueAddress>, QueryCursorError> {
        let before = budget.spent();
        if self.reject_stale(current_stamp) {
            return Ok(self.progress(QueryProgressState::Stale, before, budget));
        }
        self.cancel_active_for_navigation();
        let target = match self.selection_ordinal {
            Some(ordinal) => ordinal
                .checked_add(1)
                .ok_or(QueryCursorError::OrdinalOverflow)?,
            None => 0,
        };
        Ok(match self.ensure_ordinal(target, budget)? {
            ScannerPoll::Item { ordinal, address } => {
                self.select(ordinal, address.clone());
                self.progress(QueryProgressState::Ready(address), before, budget)
            }
            ScannerPoll::Pending => self.progress(QueryProgressState::Pending, before, budget),
            ScannerPoll::Complete => self.progress(QueryProgressState::Complete, before, budget),
        })
    }

    pub(crate) fn previous(
        &mut self,
        current_stamp: ScanRevisionStamp,
        budget: &mut WorkBudget,
    ) -> Result<QueryProgress<ValueAddress>, QueryCursorError> {
        let before = budget.spent();
        if self.reject_stale(current_stamp) {
            return Ok(self.progress(QueryProgressState::Stale, before, budget));
        }
        self.cancel_active_for_navigation();
        let Some(target) = self
            .selection_ordinal
            .and_then(|ordinal| ordinal.checked_sub(1))
        else {
            return Ok(self.progress(QueryProgressState::Complete, before, budget));
        };
        Ok(match self.ensure_ordinal(target, budget)? {
            ScannerPoll::Item { ordinal, address } => {
                self.select(ordinal, address.clone());
                self.progress(QueryProgressState::Ready(address), before, budget)
            }
            ScannerPoll::Pending => self.progress(QueryProgressState::Pending, before, budget),
            ScannerPoll::Complete => self.progress(QueryProgressState::Complete, before, budget),
        })
    }

    /// Find one adjacent result from a logical address without exposing or
    /// trusting a flattened ordinal in the caller.
    pub(crate) fn adjacent_from(
        &mut self,
        from: &ValueAddress,
        direction: CardNavigation,
        current_stamp: ScanRevisionStamp,
        budget: &mut WorkBudget,
    ) -> Result<QueryProgress<ValueAddress>, QueryCursorError> {
        let before = budget.spent();
        if self.selection != CardAddress::Value(from.clone()) {
            match self.seek(from, current_stamp, budget)?.into_state() {
                QueryProgressState::Ready(_) => {}
                state => return Ok(self.progress(state, before, budget)),
            }
        }
        let state = match direction {
            CardNavigation::Next => self.next(current_stamp, budget)?.into_state(),
            CardNavigation::Previous => self.previous(current_stamp, budget)?.into_state(),
        };
        Ok(self.progress(state, before, budget))
    }

    pub(crate) fn seek(
        &mut self,
        target: &ValueAddress,
        current_stamp: ScanRevisionStamp,
        budget: &mut WorkBudget,
    ) -> Result<QueryProgress<ValueAddress>, QueryCursorError> {
        let before = budget.spent();
        if self.reject_stale(current_stamp) {
            return Ok(self.progress(QueryProgressState::Stale, before, budget));
        }
        if self.selection == CardAddress::Value(target.clone()) {
            return Ok(self.progress(QueryProgressState::Ready(target.clone()), before, budget));
        }
        if let Some(ordinal) = self.cache.ordinal_of(target) {
            self.active = None;
            self.select(ordinal, target.clone());
            return Ok(self.progress(QueryProgressState::Ready(target.clone()), before, budget));
        }
        if !matches!(&self.active, Some(ActiveOperation::Seek(active)) if active == target) {
            self.cancel_active_for_navigation();
            self.reset_scan();
            self.active = Some(ActiveOperation::Seek(target.clone()));
        }
        loop {
            match self.poll_scanner(budget)? {
                ScannerPoll::Item { ordinal, address } if &address == target => {
                    self.active = None;
                    self.select(ordinal, address.clone());
                    return Ok(self.progress(QueryProgressState::Ready(address), before, budget));
                }
                ScannerPoll::Item { .. } => {}
                ScannerPoll::Pending => {
                    return Ok(self.progress(QueryProgressState::Pending, before, budget));
                }
                ScannerPoll::Complete => {
                    // The exhausted scanner proves this target is absent for
                    // this revision. Keep the operation so an identical retry
                    // does not discard that proof and restart the whole scan.
                    return Ok(self.progress(QueryProgressState::Complete, before, budget));
                }
            }
        }
    }

    pub(crate) fn fill_window(
        &mut self,
        anchor: Option<&ValueAddress>,
        max_cards: NonZeroUsize,
        current_stamp: ScanRevisionStamp,
        budget: &mut WorkBudget,
    ) -> Result<QueryProgress<QueryWindow>, QueryCursorError> {
        let before = budget.spent();
        let max_cards = max_cards.get();
        let required = max_cards
            .checked_add(1)
            .ok_or(QueryCursorError::OrdinalOverflow)?;
        if required > self.cache.capacity {
            return Err(QueryCursorError::WindowExceedsCache {
                requested: max_cards,
                required,
                capacity: self.cache.capacity,
            });
        }
        if self.reject_stale(current_stamp) {
            return Ok(self.progress(QueryProgressState::Stale, before, budget));
        }
        let anchor = anchor.cloned();
        let same_fill = matches!(
            &self.active,
            Some(ActiveOperation::Fill {
                anchor: active_anchor,
                max_cards: active_max,
                ..
            }) if active_anchor == &anchor && *active_max == max_cards
        );
        if !same_fill {
            self.cancel_active_for_navigation();
            let start_ordinal = match &anchor {
                Some(anchor) => self.cache.ordinal_of(anchor),
                None if self.cache.get(0).is_some() || self.scanner_next_ordinal == 0 => Some(0),
                None => {
                    self.reset_scan();
                    Some(0)
                }
            };
            if anchor.is_some() && start_ordinal.is_none() {
                self.reset_scan();
            }
            self.active = Some(ActiveOperation::Fill {
                anchor,
                max_cards,
                start_ordinal,
            });
        }

        loop {
            let (anchor, start_ordinal) = match &self.active {
                Some(ActiveOperation::Fill {
                    anchor,
                    start_ordinal,
                    ..
                }) => (anchor.clone(), *start_ordinal),
                _ => unreachable!("fill operation remains active until it returns"),
            };

            let Some(start) = start_ordinal else {
                match self.poll_scanner(budget)? {
                    ScannerPoll::Item { ordinal, address } if anchor.as_ref() == Some(&address) => {
                        if let Some(ActiveOperation::Fill { start_ordinal, .. }) = &mut self.active
                        {
                            *start_ordinal = Some(ordinal);
                        }
                    }
                    ScannerPoll::Item { .. } => {}
                    ScannerPoll::Pending => {
                        return Ok(self.progress(QueryProgressState::Pending, before, budget));
                    }
                    ScannerPoll::Complete => {
                        // Keep the exhausted fill for repeated refreshes of
                        // the same missing anchor. A different operation or
                        // revision still starts its own scan as usual.
                        return Ok(self.progress(QueryProgressState::Complete, before, budget));
                    }
                }
                continue;
            };

            let lookahead = start
                .checked_add(max_cards)
                .ok_or(QueryCursorError::OrdinalOverflow)?;
            if self.cache.contains_range(start, lookahead) {
                let window = self.window_from_cache(start, max_cards, true)?;
                self.poll_total();
                return Ok(self.progress(QueryProgressState::Ready(window), before, budget));
            }
            if self.scanner_complete {
                if start > self.scanner_next_ordinal {
                    self.active = None;
                    return Ok(self.progress(QueryProgressState::Complete, before, budget));
                }
                let start = start.min(self.scanner_next_ordinal.saturating_sub(max_cards));
                let window = self.window_from_cache(start, max_cards, false)?;
                self.poll_total();
                return Ok(self.progress(QueryProgressState::Ready(window), before, budget));
            }
            if self.scanner_next_ordinal > lookahead {
                self.reset_scan();
                if let Some(ActiveOperation::Fill {
                    anchor,
                    start_ordinal,
                    ..
                }) = &mut self.active
                {
                    *start_ordinal = anchor.is_none().then_some(0);
                }
                continue;
            }
            match self.poll_scanner(budget)? {
                ScannerPoll::Item { .. } => {}
                ScannerPoll::Pending => {
                    return Ok(self.progress(QueryProgressState::Pending, before, budget));
                }
                ScannerPoll::Complete => {}
            }
        }
    }

    pub(crate) fn end(
        &mut self,
        current_stamp: ScanRevisionStamp,
        budget: &mut WorkBudget,
    ) -> Result<QueryProgress<CardAddress>, QueryCursorError> {
        let before = budget.spent();
        if self.reject_stale(current_stamp) {
            return Ok(self.progress(QueryProgressState::Stale, before, budget));
        }
        if !matches!(self.active, Some(ActiveOperation::End { .. })) {
            self.cancel_active_for_navigation();
            self.reset_scan();
            self.total = QueryTotal::Scanning(ScanProgress::default());
            self.active = Some(ActiveOperation::End {
                last_match: None,
                matched: 0,
            });
        }
        loop {
            match self.poll_scanner(budget)? {
                ScannerPoll::Item { address, .. } => {
                    if let Some(ActiveOperation::End {
                        last_match,
                        matched,
                    }) = &mut self.active
                    {
                        *last_match = Some(address);
                        *matched = matched.saturating_add(1);
                    }
                }
                ScannerPoll::Pending => {
                    let progress = self.end_scan_progress();
                    self.total = QueryTotal::Scanning(progress);
                    return Ok(self.progress(QueryProgressState::Pending, before, budget));
                }
                ScannerPoll::Complete => {
                    let Some(ActiveOperation::End {
                        last_match,
                        matched,
                    }) = self.active.take()
                    else {
                        unreachable!("end operation remains active until exhaustion")
                    };
                    self.selection = last_match
                        .map(CardAddress::Value)
                        .unwrap_or(CardAddress::NewSlot);
                    self.selection_ordinal = matched.checked_sub(1);
                    self.total = QueryTotal::Exact(matched);
                    return Ok(self.progress(
                        QueryProgressState::Ready(self.selection.clone()),
                        before,
                        budget,
                    ));
                }
            }
        }
    }

    pub(crate) fn cancel_end(&mut self) -> QueryProgress<()> {
        let state = if matches!(self.active, Some(ActiveOperation::End { .. })) {
            self.active = None;
            self.total = QueryTotal::Unknown;
            QueryProgressState::Cancelled
        } else {
            QueryProgressState::Complete
        };
        QueryProgress::new(state, 0, self.instrumentation(), self.total)
    }

    fn end_scan_progress(&self) -> ScanProgress {
        let matched = match self.active {
            Some(ActiveOperation::End { matched, .. }) => matched,
            _ => 0,
        };
        ScanProgress {
            inspected: self.scanner.instrumentation().addressed,
            matched,
        }
    }

    fn ensure_ordinal(
        &mut self,
        target: usize,
        budget: &mut WorkBudget,
    ) -> Result<ScannerPoll, QueryCursorError> {
        if let Some(address) = self.cache.get(target).cloned() {
            return Ok(ScannerPoll::Item {
                ordinal: target,
                address,
            });
        }
        if target < self.scanner_next_ordinal {
            self.reset_scan();
        }
        loop {
            match self.poll_scanner(budget)? {
                item @ ScannerPoll::Item { ordinal, .. } if ordinal == target => return Ok(item),
                ScannerPoll::Item { .. } => {}
                other @ (ScannerPoll::Pending | ScannerPoll::Complete) => return Ok(other),
            }
        }
    }

    fn poll_scanner(&mut self, budget: &mut WorkBudget) -> Result<ScannerPoll, QueryCursorError> {
        if let Some(error) = self.scanner_error {
            return Err(error);
        }
        if self.scanner_complete {
            return Ok(ScannerPoll::Complete);
        }
        Ok(match self.scanner.poll_next(budget) {
            QueryPlanPoll::Item(address) => {
                let ordinal = self.scanner_next_ordinal;
                let Some(next_ordinal) = self.scanner_next_ordinal.checked_add(1) else {
                    // The scanner consumed a result whose ordinal cannot be
                    // represented. Keep the failure sticky until an explicit
                    // reset, so retrying cannot misreport exhausted data as complete.
                    self.scanner_error = Some(QueryCursorError::OrdinalOverflow);
                    return Err(QueryCursorError::OrdinalOverflow);
                };
                self.scanner_next_ordinal = next_ordinal;
                self.cache.insert(ordinal, address.clone());
                ScannerPoll::Item { ordinal, address }
            }
            QueryPlanPoll::Pending => ScannerPoll::Pending,
            QueryPlanPoll::Complete => {
                self.scanner_complete = true;
                self.total = QueryTotal::Exact(self.scanner_next_ordinal);
                ScannerPoll::Complete
            }
        })
    }

    fn window_from_cache(
        &self,
        start: usize,
        max_cards: usize,
        has_after: bool,
    ) -> Result<QueryWindow, QueryCursorError> {
        let limit = start
            .checked_add(max_cards)
            .ok_or(QueryCursorError::OrdinalOverflow)?;
        let end = if has_after {
            limit
        } else {
            limit.min(self.scanner_next_ordinal)
        };
        let addresses = self
            .cache
            .collect_range(start, end)
            .ok_or(QueryCursorError::MissingWindow { start, max_cards })?;
        Ok(QueryWindow::new(addresses, start, start > 0, has_after))
    }

    fn select(&mut self, ordinal: usize, address: ValueAddress) {
        self.selection = CardAddress::Value(address);
        self.selection_ordinal = Some(ordinal);
    }

    fn cancel_active_for_navigation(&mut self) {
        if matches!(self.active, Some(ActiveOperation::End { .. })) {
            self.total = QueryTotal::Unknown;
        }
        self.active = None;
    }

    fn reject_stale(&mut self, current_stamp: ScanRevisionStamp) -> bool {
        if self.stale || current_stamp != self.stamp {
            self.stale = true;
            self.active = None;
            self.cache.clear();
            self.total = QueryTotal::Unknown;
            return true;
        }
        false
    }

    fn poll_total(&mut self) {
        if matches!(self.total, QueryTotal::Exact(_)) {
            return;
        }
        // Exact totals are useful for a stable scrollbar, but this is a
        // background pass and must not consume a frame-sized chunk of work.
        let mut budget = WorkBudget::new(8);
        loop {
            match self.total_scanner.poll_next(&mut budget) {
                QueryPlanPoll::Item(_) => {
                    self.total_matched = self.total_matched.saturating_add(1);
                    let instrumentation = self.total_scanner.instrumentation();
                    self.total = QueryTotal::Scanning(ScanProgress {
                        inspected: instrumentation.addressed,
                        matched: self.total_matched,
                    });
                }
                QueryPlanPoll::Pending => return,
                QueryPlanPoll::Complete => {
                    self.total = QueryTotal::Exact(self.total_matched);
                    return;
                }
            }
        }
    }
    fn reset_scan(&mut self) {
        self.retire_current_scan();
        self.scanner = self.plan.evaluate(self.source);
        self.scanner_next_ordinal = 0;
        self.scanner_complete = false;
        self.scanner_error = None;
        self.cache.clear();
    }

    fn retire_current_scan(&mut self) {
        let current = self.scanner.instrumentation();
        self.retired.addressed = self.retired.addressed.saturating_add(current.addressed);
        self.retired.reflected = self.retired.reflected.saturating_add(current.reflected);
        self.retired.matched = self.retired.matched.saturating_add(current.matched);
        self.retired.pruned_subtrees = self
            .retired
            .pruned_subtrees
            .saturating_add(current.pruned_subtrees);
    }

    fn progress<T>(
        &self,
        state: QueryProgressState<T>,
        before: usize,
        budget: &WorkBudget,
    ) -> QueryProgress<T> {
        QueryProgress::new(
            state,
            budget.spent().saturating_sub(before),
            self.instrumentation(),
            self.total,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::arena::Arena;
    use crate::object_explorer::breadcrumb::Breadcrumb;
    use crate::object_explorer::breadcrumb::ValueFilterOperator;
    use crate::object_explorer::breadcrumbs::Breadcrumbs;
    use crate::object_explorer::query_progress::QueryProgressState;
    use crate::object_explorer::revision::QueryRevision;
    use crate::object_explorer::value_path::ValuePathSegment;
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    fn stamp(arena: &Arena) -> ScanRevisionStamp {
        ScanRevisionStamp {
            arena: arena.arena_revision(),
            query: QueryRevision::default(),
        }
    }

    fn ready<T: std::fmt::Debug>(progress: QueryProgress<T>) -> T {
        match progress.into_state() {
            QueryProgressState::Ready(value) => value,
            state => panic!("expected Ready, got {state:?}"),
        }
    }

    #[test]
    fn ordinal_exhaustion_remains_an_error_when_navigation_is_retried() {
        let mut arena = Arena::default();
        let root = arena.insert_ready(runtime(1_usize)).unwrap();
        let stamp = stamp(&arena);
        let source = ArenaAddressSource::new(&arena);
        let mut cursor = QueryCursor::new(
            &source,
            QueryPlan::new(Breadcrumbs::default()),
            stamp,
            NonZeroUsize::MIN,
        );
        cursor.select(usize::MAX - 1, ValueAddress::root(root));
        cursor.scanner_next_ordinal = usize::MAX;
        for _ in 0..2 {
            assert!(matches!(
                cursor.next(stamp, &mut WorkBudget::new(1)),
                Err(QueryCursorError::OrdinalOverflow)
            ));
            assert_eq!(
                cursor.selection(),
                &CardAddress::Value(ValueAddress::root(root))
            );
            assert_eq!(cursor.total(), QueryTotal::Unknown);
        }
        // A cached selection at the limit must fail before wrapping to zero.
        cursor.selection_ordinal = Some(usize::MAX);
        assert!(matches!(
            cursor.next(stamp, &mut WorkBudget::new(1)),
            Err(QueryCursorError::OrdinalOverflow)
        ));
    }

    #[test]
    fn million_value_first_window_has_bounded_work() {
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime((0_usize..1_000_000).collect::<Vec<_>>()))
            .unwrap();
        let stamp = stamp(&arena);
        let source = ArenaAddressSource::new(&arena);
        let mut cursor = QueryCursor::new(
            &source,
            QueryPlan::new(Breadcrumbs::default()),
            stamp,
            NonZeroUsize::new(9).unwrap(),
        );
        let mut budget = WorkBudget::new(16);

        let progress = cursor
            .fill_window(None, NonZeroUsize::new(8).unwrap(), stamp, &mut budget)
            .unwrap();
        assert_eq!(progress.work_spent(), 9, "eight cards plus one lookahead");
        assert_eq!(progress.instrumentation().addressed, 9);
        assert_eq!(progress.instrumentation().cached, 9);
        assert!(matches!(progress.total(), QueryTotal::Scanning(_)));
        let window = ready(progress);
        assert_eq!(window.addresses().len(), 8);
        assert_eq!(window.addresses()[0], ValueAddress::root(root));
        assert!(window.has_after());
        assert!(!window.has_before());
    }

    #[test]
    fn completed_tail_window_backfills_to_its_requested_capacity() {
        let mut arena = Arena::default();
        let roots = (0_usize..7)
            .map(|value| arena.insert_ready(runtime(value)).unwrap())
            .collect::<Vec<_>>();
        let stamp = stamp(&arena);
        let source = ArenaAddressSource::new(&arena);
        let mut cursor = QueryCursor::new(
            &source,
            QueryPlan::new(Breadcrumbs::default()),
            stamp,
            NonZeroUsize::new(8).unwrap(),
        );
        let mut budget = WorkBudget::new(32);

        let window = ready(
            cursor
                .fill_window(
                    Some(&ValueAddress::root(roots[6])),
                    NonZeroUsize::new(5).unwrap(),
                    stamp,
                    &mut budget,
                )
                .unwrap(),
        );

        assert_eq!(window.addresses().len(), 5);
        assert_eq!(
            window.addresses().last(),
            Some(&ValueAddress::root(roots[6]))
        );
        assert!(window.has_before());
        assert!(!window.has_after());
    }

    #[test]
    fn million_value_no_match_yields_between_budgets() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime((0_usize..1_000_000).collect::<Vec<_>>()))
            .unwrap();
        let stamp = stamp(&arena);
        let source = ArenaAddressSource::new(&arena);
        let query = Breadcrumbs::new(vec![
            Breadcrumb::ShapeFilter {
                included_shapes: vec![cloud_terrastodon_registry::describe_shape(usize::SHAPE)],
            },
            Breadcrumb::ValueFilter {
                field_shape: "*".to_owned(),
                field_name: "field_that_does_not_exist".to_owned(),
                operator: ValueFilterOperator::Equals,
                value: "never".to_owned(),
            },
        ]);
        let mut cursor = QueryCursor::new(
            &source,
            QueryPlan::new(query),
            stamp,
            NonZeroUsize::new(16).unwrap(),
        );

        let mut first_budget = WorkBudget::new(7);
        let first = cursor.next(stamp, &mut first_budget).unwrap();
        assert_eq!(first.state(), &QueryProgressState::Pending);
        assert_eq!(first.work_spent(), 7);
        assert_eq!(first.instrumentation().addressed, 7);
        assert_eq!(first.instrumentation().matched, 0);
        assert_eq!(cursor.selection(), &CardAddress::NewSlot);
        assert_eq!(cursor.total(), QueryTotal::Unknown);

        let mut second_budget = WorkBudget::new(11);
        let second = cursor.next(stamp, &mut second_budget).unwrap();
        assert_eq!(second.state(), &QueryProgressState::Pending);
        assert_eq!(second.work_spent(), 11);
        assert_eq!(second.instrumentation().addressed, 18);
        assert_eq!(second.instrumentation().matched, 0);
    }

    #[test]
    fn missing_anchor_retries_remain_complete_until_navigation_changes() {
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime((0_usize..16).collect::<Vec<_>>()))
            .unwrap();
        let excluded = arena
            .insert_ready(runtime(String::from("new request")))
            .unwrap();
        let stamp = stamp(&arena);
        let source = ArenaAddressSource::object_pool(&arena);
        let mut cursor = QueryCursor::new(
            &source,
            QueryPlan::new(Breadcrumbs::new(vec![Breadcrumb::projection(
                root.get(),
                vec![],
            )])),
            stamp,
            NonZeroUsize::new(4).unwrap(),
        );
        let missing = ValueAddress::root(excluded);

        // A newly created root lies outside the tab's existing projection.
        // More matches than cache entries force its absence to be established
        // by completing the bounded scan, not by inspecting the current page.
        let mut completed = false;
        for _ in 0..10 {
            let progress = cursor
                .fill_window(
                    Some(&missing),
                    NonZeroUsize::new(2).unwrap(),
                    stamp,
                    &mut WorkBudget::new(3),
                )
                .unwrap();
            if matches!(progress.state(), QueryProgressState::Complete) {
                completed = true;
                break;
            }
            assert_eq!(progress.state(), &QueryProgressState::Pending);
        }
        assert!(completed);
        let instrumentation = cursor.instrumentation();
        assert_eq!(instrumentation.matched, 17);
        assert!(instrumentation.cached <= 4);
        for _ in 0..3 {
            let progress = cursor
                .fill_window(
                    Some(&missing),
                    NonZeroUsize::new(2).unwrap(),
                    stamp,
                    &mut WorkBudget::new(3),
                )
                .unwrap();
            assert_eq!(progress.state(), &QueryProgressState::Complete);
            assert_eq!(progress.work_spent(), 0);
            assert_eq!(progress.instrumentation(), instrumentation);
        }

        let window = ready(
            cursor
                .fill_window(
                    None,
                    NonZeroUsize::new(2).unwrap(),
                    stamp,
                    &mut WorkBudget::new(3),
                )
                .unwrap(),
        );
        assert_eq!(window.addresses()[0], ValueAddress::root(root));
        assert!(window.has_after());

        assert_eq!(
            cursor
                .seek(&missing, stamp, &mut WorkBudget::new(3))
                .unwrap()
                .state(),
            &QueryProgressState::Pending,
        );
        assert_eq!(
            cursor
                .seek(&missing, stamp, &mut WorkBudget::new(32))
                .unwrap()
                .state(),
            &QueryProgressState::Complete,
        );
        let instrumentation = cursor.instrumentation();
        for _ in 0..3 {
            let progress = cursor
                .seek(&missing, stamp, &mut WorkBudget::new(3))
                .unwrap();
            assert_eq!(progress.state(), &QueryProgressState::Complete);
            assert_eq!(progress.work_spent(), 0);
            assert_eq!(progress.instrumentation(), instrumentation);
        }

        assert_eq!(
            ready(
                cursor
                    .seek(&ValueAddress::root(root), stamp, &mut WorkBudget::new(3))
                    .unwrap()
            ),
            ValueAddress::root(root),
        );
        assert_eq!(
            ready(cursor.next(stamp, &mut WorkBudget::new(3)).unwrap()),
            ValueAddress::root(root).child(ValuePathSegment::Index(0)),
        );
    }

    #[test]
    fn next_previous_seek_and_cache_are_bounded_and_address_based() {
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime(vec![10_usize, 20, 30, 40, 50, 60]))
            .unwrap();
        let stamp = stamp(&arena);
        let source = ArenaAddressSource::new(&arena);
        let mut cursor = QueryCursor::new(
            &source,
            QueryPlan::new(Breadcrumbs::default()),
            stamp,
            NonZeroUsize::new(4).unwrap(),
        );

        let mut one = WorkBudget::new(1);
        assert_eq!(
            ready(cursor.next(stamp, &mut one).unwrap()),
            ValueAddress::root(root)
        );
        let first_child = ValueAddress::root(root).child(ValuePathSegment::Index(0));
        let mut one = WorkBudget::new(1);
        assert_eq!(ready(cursor.next(stamp, &mut one).unwrap()), first_child);

        let mut no_work = WorkBudget::new(0);
        assert_eq!(
            ready(cursor.previous(stamp, &mut no_work).unwrap()),
            ValueAddress::root(root),
            "previous uses the bounded cache without scanning"
        );

        let target = ValueAddress::root(root).child(ValuePathSegment::Index(4));
        let mut partial = WorkBudget::new(3);
        assert_eq!(
            cursor.seek(&target, stamp, &mut partial).unwrap().state(),
            &QueryProgressState::Pending
        );
        assert_eq!(
            cursor.selection(),
            &CardAddress::Value(ValueAddress::root(root))
        );
        let mut rest = WorkBudget::new(3);
        assert_eq!(
            ready(cursor.seek(&target, stamp, &mut rest).unwrap()),
            target
        );
        assert!(cursor.instrumentation().cached <= 4);
    }

    #[test]
    fn end_is_cooperative_atomic_cancellable_and_revision_scoped() {
        let mut arena = Arena::default();
        let root = arena.insert_ready(runtime(vec![1_usize, 2, 3, 4])).unwrap();
        let stamp = stamp(&arena);
        let source = ArenaAddressSource::new(&arena);
        let mut cursor = QueryCursor::new(
            &source,
            QueryPlan::new(Breadcrumbs::default()),
            stamp,
            NonZeroUsize::new(3).unwrap(),
        );
        let mut select_budget = WorkBudget::new(2);
        let selected = ready(
            cursor
                .seek(
                    &ValueAddress::root(root).child(ValuePathSegment::Index(0)),
                    stamp,
                    &mut select_budget,
                )
                .unwrap(),
        );

        let mut partial = WorkBudget::new(2);
        assert_eq!(
            cursor.end(stamp, &mut partial).unwrap().state(),
            &QueryProgressState::Pending
        );
        assert_eq!(cursor.selection(), &CardAddress::Value(selected.clone()));
        assert!(matches!(cursor.total(), QueryTotal::Scanning(_)));
        assert_eq!(cursor.cancel_end().state(), &QueryProgressState::Cancelled);
        assert_eq!(cursor.selection(), &CardAddress::Value(selected.clone()));
        assert_eq!(cursor.total(), QueryTotal::Unknown);

        let mut first = WorkBudget::new(2);
        assert_eq!(
            cursor.end(stamp, &mut first).unwrap().state(),
            &QueryProgressState::Pending
        );
        let stale_stamp = ScanRevisionStamp {
            arena: stamp.arena,
            query: stamp.query.next().unwrap(),
        };
        let mut stale_budget = WorkBudget::new(10);
        assert_eq!(
            cursor.end(stale_stamp, &mut stale_budget).unwrap().state(),
            &QueryProgressState::Stale
        );
        assert_eq!(cursor.selection(), &CardAddress::Value(selected));
        assert_eq!(cursor.total(), QueryTotal::Unknown);

        let source = ArenaAddressSource::new(&arena);
        let mut complete_cursor = QueryCursor::new(
            &source,
            QueryPlan::new(Breadcrumbs::default()),
            stamp,
            NonZeroUsize::new(3).unwrap(),
        );
        let mut enough = WorkBudget::new(6);
        let result = ready(complete_cursor.end(stamp, &mut enough).unwrap());
        assert_eq!(
            result,
            CardAddress::Value(ValueAddress::root(root).child(ValuePathSegment::Index(3)))
        );
        assert_eq!(complete_cursor.total(), QueryTotal::Exact(5));
    }

    #[test]
    fn instrumentation_tracks_serialization_without_retaining_values() {
        let mut arena = Arena::default();
        arena.insert_ready(runtime(String::from("one"))).unwrap();
        let stamp = stamp(&arena);
        let source = ArenaAddressSource::new(&arena);
        let mut cursor = QueryCursor::new(
            &source,
            QueryPlan::new(Breadcrumbs::default()),
            stamp,
            NonZeroUsize::new(2).unwrap(),
        );
        let mut budget = WorkBudget::new(1);
        let _ = ready(cursor.next(stamp, &mut budget).unwrap());
        cursor.record_serialized(1);

        let instrumentation = cursor.instrumentation();
        assert_eq!(instrumentation.addressed, 1);
        assert_eq!(instrumentation.matched, 1);
        assert_eq!(instrumentation.serialized, 1);
        assert_eq!(instrumentation.cached, 1);
        assert!(instrumentation.reflected >= 1);
    }
}
