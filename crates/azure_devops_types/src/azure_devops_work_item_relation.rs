use crate::AzureDevOpsWorkItemRelationAttributes;
use crate::AzureDevOpsWorkItemRelationTypeName;
use crate::AzureDevOpsWorkItemRelationUrl;

pub const WORK_ITEM_CHILD_RELATION: &str = "System.LinkTypes.Hierarchy-Forward";
pub const WORK_ITEM_PARENT_RELATION: &str = "System.LinkTypes.Hierarchy-Reverse";

#[derive(Debug, Clone, PartialEq, Eq, arbitrary::Arbitrary, facet::Facet)]
pub struct AzureDevOpsWorkItemRelation {
    pub rel: AzureDevOpsWorkItemRelationTypeName,
    pub url: AzureDevOpsWorkItemRelationUrl,
    #[facet(default)]
    pub attributes: AzureDevOpsWorkItemRelationAttributes,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemRelation);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemRelation);
