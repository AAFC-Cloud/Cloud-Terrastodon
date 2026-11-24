use crate::portal_tab_id::PortalTabId;

/// Remembers the selected tab and rail expansion independently for each pane.
pub(crate) struct PortalPane {
    pub active: PortalTabId,
    pub expanded: bool,
}

impl PortalPane {
    pub fn new(active: PortalTabId) -> Self {
        Self {
            active,
            expanded: true,
        }
    }
}
