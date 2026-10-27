use crate::CancellationScope;
use crate::DiscoveryPlanEdge;
use crate::DiscoveryPlanNode;
use crate::Function;
use crate::discovery_plan_graph::DiscoveryPlanGraph;
use facet::Facet;
use facet::Shape;

/// An offline, target-directed graph of structurally possible discovery work.
///
/// Planning inspects only the supplied catalog and actual Facet shapes. It never
/// invokes requests, constructors, conversions, defaults, or credential lookup.
/// Seeds are opaque: supplying an enum does not supply all its variant payloads.
/// Optional request fields may be omitted and cancellation is host-supplied.
/// Required domain values must be seeded, returned, or converted explicitly.
/// Requests with no required domain inputs (including those with only optional
/// fields and cancellation) require their input shape to be explicitly supplied;
/// they cannot bootstrap discovery through ambient configuration or defaults.
///
/// Reachability is potential type coverage: empty collections, absent options,
/// inactive variants, failed conversions, and API errors can prevent live work.
/// Shape compatibility does not prove organization/tenant identity correlation.
/// This graph is therefore a plan for inspection, not an executable schedule.
/// For reachable targets it retains one deterministic derivation with every
/// required input. For blocked targets it retains candidate producers and their
/// unmet dependencies. Unrelated catalog branches are excluded.
#[derive(Debug, Clone, Facet)]
pub struct DiscoveryPlan {
    pub nodes: Vec<DiscoveryPlanNode>,
    pub edges: Vec<DiscoveryPlanEdge>,
    pub diagnostics: Vec<String>,
}

impl DiscoveryPlan {
    /// Build a finite structural closure, then retain target dependencies.
    ///
    /// Only unattended reads and pure synchronous conversions/projections are
    /// admitted. Catalog effects remain declarations requiring an audit. No
    /// global registry is consulted, including for Arbitrary generators.
    pub fn build(
        functions: impl IntoIterator<Item = &'static Function>,
        seeds: impl IntoIterator<Item = &'static Shape>,
        targets: impl IntoIterator<Item = &'static Shape>,
        cancellation: &CancellationScope,
    ) -> eyre::Result<Self> {
        DiscoveryPlanGraph::new(functions, seeds, targets, cancellation)?.finish(cancellation)
    }
}

crate::register_thing!(DiscoveryPlan);
