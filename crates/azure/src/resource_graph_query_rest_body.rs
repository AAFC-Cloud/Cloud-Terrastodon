use crate::ResourceGraphQueryRestOptions;
use facet::Facet;

/// [Resource Graph query request](https://learn.microsoft.com/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01#queryrequest).
#[derive(Debug, Facet)]
pub struct ResourceGraphQueryRestBody {
    pub query: String,
    pub options: ResourceGraphQueryRestOptions,
}
