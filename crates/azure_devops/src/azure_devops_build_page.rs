use facet::Facet;

/// Typed value envelope shared by Azure DevOps Build list responses.
///
/// See Microsoft's [Builds List API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1).
/// Its envelope follows the documented [REST response format](https://learn.microsoft.com/en-us/azure/devops/integrate/how-to/call-rest-api?view=azure-devops#response-format).
#[derive(Debug, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub(crate) struct AzureDevOpsBuildPage<T> {
    // The service's count metadata does not replace continuation headers.
    pub(crate) count: usize,
    pub(crate) value: Vec<T>,
}
