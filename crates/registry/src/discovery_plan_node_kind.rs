use facet::Facet;

/// The role of a node in a structural discovery plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Facet)]
#[repr(u8)]
pub enum DiscoveryPlanNodeKind {
    Shape,
    Request,
    Conversion,
    Projection,
}
