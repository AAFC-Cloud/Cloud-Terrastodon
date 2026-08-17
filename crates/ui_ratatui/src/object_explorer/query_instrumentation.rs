use super::query_plan::QueryPlanInstrumentation;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct QueryInstrumentation {
    pub(crate) addressed: usize,
    pub(crate) reflected: usize,
    pub(crate) matched: usize,
    pub(crate) serialized: usize,
    pub(crate) cached: usize,
    pub(crate) pruned_subtrees: usize,
}

impl QueryInstrumentation {
    pub(crate) fn from_plan(
        retired: QueryPlanInstrumentation,
        current: QueryPlanInstrumentation,
        serialized: usize,
        cached: usize,
    ) -> Self {
        Self {
            addressed: retired.addressed.saturating_add(current.addressed),
            reflected: retired.reflected.saturating_add(current.reflected),
            matched: retired.matched.saturating_add(current.matched),
            serialized,
            cached,
            pruned_subtrees: retired
                .pruned_subtrees
                .saturating_add(current.pruned_subtrees),
        }
    }
}
