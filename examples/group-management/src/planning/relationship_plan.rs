use crate::domain::ObservedPrincipal;
use crate::domain::ReconciliationMode;
use facet::Facet;

/// The baseline, resolved target, and proposed differences for one direct list.
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
#[facet(rename_all = "camelCase")]
pub struct RelationshipPlan {
    pub mode: ReconciliationMode,
    pub inventory_complete: bool,
    pub observed: Vec<ObservedPrincipal>,
    pub desired: Vec<ObservedPrincipal>,
    pub add: Vec<ObservedPrincipal>,
    pub remove: Vec<ObservedPrincipal>,
}
