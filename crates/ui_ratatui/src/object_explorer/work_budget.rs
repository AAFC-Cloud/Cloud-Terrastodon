/// Maximum number of query advancement attempts allowed in one cooperative turn.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorkBudget {
    limit: usize,
    spent: usize,
}

impl WorkBudget {
    pub(crate) const fn new(limit: usize) -> Self {
        Self { limit, spent: 0 }
    }

    pub(crate) fn try_consume(&mut self) -> bool {
        if self.spent == self.limit {
            return false;
        }
        self.spent += 1;
        true
    }

    pub(crate) const fn spent(self) -> usize {
        self.spent
    }

    pub(crate) const fn limit(self) -> usize {
        self.limit
    }
}
