use facet::Facet;
use std::sync::Arc;
use url::Url;

/// OData collection envelope returned by the legacy Graph PIM endpoints.
/// See [Graph paging](https://learn.microsoft.com/graph/paging) for continuation
/// links and [role definitions](https://learn.microsoft.com/graph/api/resources/governanceroledefinition?view=graph-rest-beta)
/// for the contained PIM records. Metadata and continuation are omitted when absent.
#[derive(Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct EntraPimCollectionResponse<T> {
    pub value: Vec<Arc<T>>,
    #[facet(rename = "@odata.context")]
    pub context: Option<String>,
    #[facet(rename = "@odata.nextLink")]
    pub next_link: Option<Url>,
}
