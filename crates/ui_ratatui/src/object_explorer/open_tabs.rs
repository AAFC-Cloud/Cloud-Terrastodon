use super::slot_id::SlotId;

/// Non-owning UI order for ordinary arena-owned Tab values.
#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct OpenTabs {
    tabs: Vec<SlotId>,
    active: Option<usize>,
}

impl OpenTabs {
    pub(crate) fn open(&mut self, tab: SlotId) {
        if let Some(index) = self.tabs.iter().position(|candidate| *candidate == tab) {
            self.active = Some(index);
            return;
        }
        self.tabs.push(tab);
        self.active = Some(self.tabs.len() - 1);
    }

    pub(crate) fn close(&mut self, tab: SlotId) -> bool {
        let Some(index) = self.tabs.iter().position(|candidate| *candidate == tab) else {
            return false;
        };
        let previous_active = self.active;
        self.tabs.remove(index);
        self.active = match (self.tabs.len(), previous_active) {
            (0, _) => None,
            (_, None) => None,
            (_, Some(active)) if active == index => Some(index.saturating_sub(1)),
            (_, Some(active)) if active > index => Some(active - 1),
            (_, Some(active)) => Some(active),
        };
        true
    }

    pub(crate) fn activate(&mut self, tab: SlotId) -> bool {
        let Some(index) = self.tabs.iter().position(|candidate| *candidate == tab) else {
            return false;
        };
        self.active = Some(index);
        true
    }

    pub(crate) fn active(&self) -> Option<SlotId> {
        self.active.and_then(|index| self.tabs.get(index).copied())
    }

    pub(crate) fn previous(&self) -> Option<SlotId> {
        self.active
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| self.tabs.get(index).copied())
    }

    pub(crate) fn next(&self) -> Option<SlotId> {
        self.active
            .and_then(|index| index.checked_add(1))
            .and_then(|index| self.tabs.get(index).copied())
    }

    pub(crate) fn ids(&self) -> &[SlotId] {
        &self.tabs
    }

    pub(crate) fn active_ordinal(&self) -> Option<(usize, usize)> {
        self.active
            .map(|index| (index.saturating_add(1), self.tabs.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_tabs_are_non_owning_slot_id_references() {
        let first = SlotId::new(2);
        let second = SlotId::new(9);
        let mut tabs = OpenTabs::default();

        tabs.open(first);
        tabs.open(second);

        assert_eq!(tabs.ids(), [first, second]);
        assert_eq!(tabs.active(), Some(second));
        assert_eq!(tabs.active_ordinal(), Some((2, 2)));
        assert_eq!(
            std::mem::size_of_val(&tabs),
            std::mem::size_of::<Vec<SlotId>>() + std::mem::size_of::<Option<usize>>(),
            "OpenTabs contains navigation identities, not RuntimeValue copies"
        );

        assert!(tabs.close(second));
        assert_eq!(tabs.active(), Some(first));
        assert_eq!(tabs.active_ordinal(), Some((1, 1)));
    }

    #[test]
    fn opening_an_existing_tab_changes_navigation_without_duplicating_it() {
        let first = SlotId::new(2);
        let second = SlotId::new(9);
        let mut tabs = OpenTabs::default();
        tabs.open(first);
        tabs.open(second);

        tabs.open(first);

        assert_eq!(tabs.ids(), [first, second]);
        assert_eq!(tabs.active(), Some(first));
    }

    #[test]
    fn closing_an_inactive_tab_preserves_the_active_tab_identity() {
        let first = SlotId::new(2);
        let second = SlotId::new(9);
        let third = SlotId::new(12);
        let mut tabs = OpenTabs::default();
        tabs.open(first);
        tabs.open(second);
        tabs.open(third);

        assert!(tabs.close(first));

        assert_eq!(tabs.ids(), [second, third]);
        assert_eq!(tabs.active(), Some(third));
        assert!(!tabs.activate(first));
        assert!(tabs.activate(second));
        assert_eq!(tabs.active(), Some(second));
    }
}
