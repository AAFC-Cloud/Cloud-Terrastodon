/// The selected sort key and direction of a reflected inventory table.
#[derive(Clone, Copy)]
pub(crate) struct ReflectedTableSort {
    pub column: usize,
    pub descending: bool,
}

impl ReflectedTableSort {
    pub const INITIAL: Self = Self {
        column: 0,
        descending: false,
    };

    pub fn select(&mut self, column: usize) {
        self.descending = self.column == column && !self.descending;
        self.column = column;
    }

    pub fn indicator(self, column: usize) -> &'static str {
        if self.column != column {
            "↕"
        } else if self.descending {
            "↓"
        } else {
            "↑"
        }
    }
}
