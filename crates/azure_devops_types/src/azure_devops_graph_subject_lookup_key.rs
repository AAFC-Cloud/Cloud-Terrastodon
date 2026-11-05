use crate::AzureDevOpsDescriptor;

/// A descriptor selected by [Graph subject lookup](https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/subject-lookup/lookup-subjects?view=azure-devops-rest-7.1#graphsubjectlookupkey).
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsGraphSubjectLookupKey {
    pub descriptor: AzureDevOpsDescriptor,
}
