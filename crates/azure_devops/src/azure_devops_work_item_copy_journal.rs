use crate::azure_devops_work_item_copy_mapping::AzureDevOpsWorkItemCopyMapping;
use crate::azure_devops_work_item_copy_plan::AzureDevOpsWorkItemCopyPlan;
use facet::Facet;

#[derive(Debug, Clone, arbitrary::Arbitrary, Facet)]
pub struct AzureDevOpsWorkItemCopyJournal {
    pub version: u32,
    pub plan: AzureDevOpsWorkItemCopyPlan,
    pub created: Vec<AzureDevOpsWorkItemCopyMapping>,
    pub completed_relations: usize,
    /// Persisted before sending a write. An uncertain write must be reconciled
    /// manually instead of automatically repeating a possibly committed POST.
    pub in_flight: Option<String>,
    pub complete: bool,
}
