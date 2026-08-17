#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct BreadcrumbBarFocus {
    position: usize,
}

impl BreadcrumbBarFocus {
    pub(super) const fn add(breadcrumb_count: usize) -> Self {
        Self {
            position: breadcrumb_count,
        }
    }

    pub(super) const fn first() -> Self {
        Self { position: 0 }
    }

    pub(super) const fn position(self) -> usize {
        self.position
    }

    pub(super) const fn operation(self, breadcrumb_count: usize) -> Option<usize> {
        if self.position < breadcrumb_count {
            Some(self.position)
        } else {
            None
        }
    }

    pub(super) const fn is_add(self, breadcrumb_count: usize) -> bool {
        self.position >= breadcrumb_count
    }

    pub(super) fn move_previous(&mut self) {
        self.position = self.position.saturating_sub(1);
    }

    pub(super) fn move_next(&mut self, breadcrumb_count: usize) {
        self.position = self.position.saturating_add(1).min(breadcrumb_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breadcrumb_focus_moves_between_operations_and_add_without_ordinals_leaking() {
        let mut focus = BreadcrumbBarFocus::add(2);
        assert!(focus.is_add(2));

        focus.move_previous();
        assert_eq!(focus.operation(2), Some(1));
        focus.move_previous();
        assert_eq!(focus.operation(2), Some(0));
        focus.move_previous();
        assert_eq!(focus.operation(2), Some(0));

        focus.move_next(2);
        focus.move_next(2);
        assert!(focus.is_add(2));
    }
}
