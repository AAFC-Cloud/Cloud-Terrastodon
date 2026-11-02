use crate::ResourceGraphData;
use facet::Facet;

/// Resource Graph's query envelope when the caller requests table data.
///
/// See the [query response schema](https://learn.microsoft.com/en-us/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01#queryresponse)
/// for API version 2022-10-01. Row decoding is a separate conversion to
/// [`crate::ResourceGraphQueryResponse`].
#[derive(Debug, Clone, PartialEq, Facet)]
pub struct RawResourceGraphQueryResponse {
    pub count: u64,
    pub data: ResourceGraphData,
    #[facet(rename = "$skipToken")]
    pub skip_token: Option<String>,
    #[facet(rename = "resultTruncated")]
    pub truncated: String,
    #[facet(rename = "totalRecords")]
    pub total_records: u64,
}
