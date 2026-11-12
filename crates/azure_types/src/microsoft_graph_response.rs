use arbitrary::Arbitrary;
use arbitrary::Unstructured;
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

impl<'a, T: Arbitrary<'a>> Arbitrary<'a> for MicrosoftGraphResponse<T> {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let value = Vec::<T>::arbitrary(input)?;
        // Generate a complete offline page. Continuation URLs belong to a live
        // query, and a supplied count must agree with this generated inventory.
        Ok(Self {
            context: None,
            count: input.arbitrary::<bool>()?.then_some(value.len()),
            next_link: None,
            value,
        })
    }
}
