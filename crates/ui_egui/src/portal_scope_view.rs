use facet::Facet;

/// A destination within the selected Azure scope's workspace.
#[derive(Clone, Copy, Debug, Eq, Facet, PartialEq, Hash)]
#[repr(u8)]
pub enum PortalScopeView {
    ActivityLog,
    AccessControl,
    Tags,
    StorageExplorer,
    Networking,
}

impl PortalScopeView {
    pub fn label(self) -> &'static str {
        match self {
            Self::ActivityLog => "Activity log",
            Self::AccessControl => "Access control (IAM)",
            Self::Tags => "Tags",
            Self::StorageExplorer => "Storage Explorer",
            Self::Networking => "Networking",
        }
    }
}
