use facet::Facet;
use std::collections::HashMap;

/// Aggregate findings from the Azure DevOps license and test-plan audit.
#[derive(Debug, Facet)]
pub struct AzureDevOpsAuditSummary {
    pub total_problems: usize,
    pub total_cost_waste_cad: f64,
    pub message_counts: HashMap<String, usize>,
}
