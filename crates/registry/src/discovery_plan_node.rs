use crate::DiscoveryPlanField;
use crate::DiscoveryPlanNodeKind;
use crate::Effect;
use facet::Facet;

/// A shape or registered operation retained in a target's dependency graph.
#[derive(Debug, Clone, Facet)]
pub struct DiscoveryPlanNode {
    /// Index in the containing plan's node list.
    pub id: usize,
    pub label: String,
    pub kind: DiscoveryPlanNodeKind,
    /// Structural possibility, not proof of nonempty data or request success.
    pub reachable: bool,
    pub seed: bool,
    pub target: bool,
    /// Depth of the selected structural derivation; zero for blocked nodes.
    pub level: usize,
    pub effects: Vec<Effect>,
    /// Direct fields of a shape or executable request, with declared types.
    #[facet(default)]
    pub fields: Vec<DiscoveryPlanField>,
    /// Required inputs that cannot be obtained from the supplied seeds/catalog.
    pub missing_inputs: Vec<String>,
}
