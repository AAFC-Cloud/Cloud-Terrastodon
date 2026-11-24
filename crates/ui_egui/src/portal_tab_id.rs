/// A tab's identity across drag-and-drop, reordering, and selection changes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PortalTabId(pub u64);
