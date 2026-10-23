/// A measured node's final rectangle in a discovery-plan layout.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DiscoveryPlanLayoutNode {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
