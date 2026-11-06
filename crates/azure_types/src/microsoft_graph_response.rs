use facet::Facet;
use url::Url;

/// A Microsoft Graph collection page.
/// [Paging Microsoft Graph data](https://learn.microsoft.com/graph/paging).
#[derive(Debug, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct MicrosoftGraphResponse<T> {
    /// Metadata may be omitted when the request negotiates no OData metadata.
    #[facet(rename = "@odata.context")]
    pub context: Option<Url>,
    #[facet(rename = "@odata.count")]
    pub count: Option<usize>,
    #[facet(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    pub value: Vec<T>,
}
