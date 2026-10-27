use facet::Facet;

/// Optional presentation hint for a frame whose grouped text rows need native
/// browser measurement before placement. The VS Code renderer measures wrapping,
/// keeps the newest groups that fit, and leaves semantic controls outside these
/// groups alone. Visible labels still identify controls without this hint.
#[derive(Debug, Clone, Facet)]
pub(crate) struct ExcalidrawRowLayout {
    pub groups: Vec<String>,
    /// Insets from the frame's top and bottom edges, in canvas coordinates.
    pub top: f64,
    pub bottom: f64,
    pub gap: f64,
}
