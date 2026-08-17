use crate::object_explorer::{
    FieldCandidateAction, FieldCandidateActions, FieldCandidateConsequence,
};

/// Picker-local selection over engine-validated transfer consequences.
pub(crate) struct LinkActionPicker {
    actions: FieldCandidateActions,
    selected: usize,
}

impl LinkActionPicker {
    pub(crate) fn new(actions: FieldCandidateActions) -> Self {
        Self {
            actions,
            selected: 0,
        }
    }

    pub(crate) fn consequences(&self) -> &[FieldCandidateConsequence] {
        self.actions.consequences()
    }

    pub(crate) const fn selected_index(&self) -> usize {
        self.selected
    }

    pub(crate) fn selected(&self) -> Option<&FieldCandidateConsequence> {
        self.actions.consequences().get(self.selected)
    }

    pub(crate) fn selected_action(&self) -> Option<FieldCandidateAction> {
        self.selected().map(FieldCandidateConsequence::action)
    }

    pub(crate) fn move_next(&mut self) {
        if !self.actions.consequences().is_empty() {
            self.selected = (self.selected + 1).min(self.actions.consequences().len() - 1);
        }
    }

    pub(crate) fn move_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}
