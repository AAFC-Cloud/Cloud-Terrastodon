use crate::azure_devops_work_item_copy_edge::AzureDevOpsWorkItemCopyEdge;
use crate::azure_devops_work_item_copy_node::AzureDevOpsWorkItemCopyNode;
use crate::azure_devops_work_item_copy_omission::AzureDevOpsWorkItemCopyOmission;
use chrono::{DateTime, Utc};
use cloud_terrastodon_azure_devops_types::{
    AzureDevOpsOrganizationUrl, AzureDevOpsProjectArgument, AzureDevOpsWorkItemId,
};
use facet::Facet;

#[derive(Debug, Clone, arbitrary::Arbitrary, Facet)]
pub struct AzureDevOpsWorkItemCopyPlan {
    pub root: AzureDevOpsWorkItemId,
    pub organization: AzureDevOpsOrganizationUrl,
    pub project: AzureDevOpsProjectArgument<'static>,
    pub as_of: DateTime<Utc>,
    pub nodes: Vec<AzureDevOpsWorkItemCopyNode>,
    pub deferred_relations: Vec<AzureDevOpsWorkItemCopyEdge>,
    pub omissions: Vec<AzureDevOpsWorkItemCopyOmission>,
}
