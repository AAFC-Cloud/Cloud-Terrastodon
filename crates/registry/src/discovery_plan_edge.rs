use facet::Facet;

/// A required input, registered output, or reflected descendant relationship.
#[derive(Debug, Clone, Facet)]
pub struct DiscoveryPlanEdge {
    pub source: usize,
    pub target: usize,
    pub label: String,
}
