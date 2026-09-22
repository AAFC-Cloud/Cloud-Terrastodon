use crate::AzureDevOpsWorkItemQuery;

pub struct AzureDevOpsWorkItemQueryFlattenedHierarchyEntry<'a> {
    pub parents: Vec<&'a AzureDevOpsWorkItemQuery>,
    pub child: &'a AzureDevOpsWorkItemQuery,
}
