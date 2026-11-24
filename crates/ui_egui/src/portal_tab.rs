use crate::portal_pane_id::PortalPaneId;
use crate::portal_tab_group_id::PortalTabGroupId;
use crate::portal_tab_id::PortalTabId;
use crate::portal_tab_target::PortalTabTarget;

const MAX_VISITS: usize = 64;

/// A tab's visits, active destination, and preview replacement policy.
#[derive(Clone)]
pub(crate) struct PortalTab {
    visits: Vec<PortalTabTarget>,
    position: usize,
    pub id: PortalTabId,
    pub pane: PortalPaneId,
    pub preview: bool,
    pub pinned: bool,
    pub group: Option<PortalTabGroupId>,
}

impl PortalTab {
    pub fn new(id: PortalTabId, target: PortalTabTarget, preview: bool) -> Self {
        Self {
            visits: vec![target],
            position: 0,
            id,
            pane: PortalPaneId::default(),
            preview,
            pinned: false,
            group: None,
        }
    }

    pub fn target(&self) -> &PortalTabTarget {
        &self.visits[self.position]
    }

    pub fn history(&self) -> (&[PortalTabTarget], usize) {
        (&self.visits, self.position)
    }

    pub fn navigate(&mut self, target: PortalTabTarget) -> bool {
        if self.target() == &target {
            return false;
        }
        self.visits.truncate(self.position + 1);
        if self.visits.len() == MAX_VISITS {
            self.visits.remove(0);
        }
        self.visits.push(target);
        self.position = self.visits.len() - 1;
        true
    }

    /// Replace the displayed visit while preserving the back and forward history.
    pub fn replace_current(&mut self, target: PortalTabTarget) {
        self.visits[self.position] = target;
    }

    pub fn go_to(&mut self, position: usize) -> bool {
        if position >= self.visits.len() || position == self.position {
            return false;
        }
        self.position = position;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_evicts_oldest_visits_at_capacity() {
        let mut tab = PortalTab::new(
            PortalTabId(0),
            PortalTabTarget::LocalPath("origin".into()),
            true,
        );
        for index in 0..MAX_VISITS {
            tab.navigate(PortalTabTarget::LocalPath(index.to_string().into()));
        }
        let (visits, position) = tab.history();
        assert_eq!(visits.len(), MAX_VISITS);
        assert_eq!(position, MAX_VISITS - 1);
        assert_eq!(
            visits.first(),
            Some(&PortalTabTarget::LocalPath("0".into()))
        );
    }
}
