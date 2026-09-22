/// Attributes accepted when creating or updating an Azure DevOps work item
/// relation.
#[derive(Debug, Clone, Default, PartialEq, Eq, arbitrary::Arbitrary, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationInputAttributes {
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub comment: Option<String>,
    #[facet(
        rename = "isLocked",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub is_locked: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemRelationInputAttributes);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemRelationInputAttributes);
