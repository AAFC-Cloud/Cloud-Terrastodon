use crate::AzureDevOpsGraphSubjectLookupKey;

/// Request body for [Graph subject lookup](https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/subject-lookup/lookup-subjects?view=azure-devops-rest-7.1#request-body).
#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsGraphSubjectLookupBody {
    pub lookup_keys: Vec<AzureDevOpsGraphSubjectLookupKey>,
}
