/// Azure DevOps' count/value envelope for a collection or descriptor dictionary.
///
/// See the [repository list response](https://learn.microsoft.com/en-us/rest/api/azure/devops/git/repositories/list?view=azure-devops-rest-7.1#examples)
/// and [subject lookup dictionary](https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/subject-lookup/lookup-subjects?view=azure-devops-rest-7.1#examples).
#[derive(Debug, Clone, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsCollectionResponse<T> {
    pub count: u32,
    pub value: T,
}
