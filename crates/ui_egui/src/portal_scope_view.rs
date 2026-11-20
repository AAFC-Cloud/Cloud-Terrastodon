/// A destination within the selected Azure scope's workspace.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PortalScopeView {
    ActivityLog,
    AccessControl,
}

impl PortalScopeView {
    pub fn label(self) -> &'static str {
        match self {
            Self::ActivityLog => "Activity log",
            Self::AccessControl => "Access control (IAM)",
        }
    }
}
