/// An orthogonal route with a reserved, top-left text anchor.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DiscoveryPlanLayoutEdge {
    pub points: Vec<[f64; 2]>,
    pub label_x: f64,
    pub label_y: f64,
}
