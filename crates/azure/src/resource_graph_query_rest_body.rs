use crate::ResourceGraphQueryRestOptions;

/// [Resource Graph query request](https://learn.microsoft.com/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01#queryrequest).
#[derive(Debug, facet::Facet)]
pub struct ResourceGraphQueryRestBody {
    pub query: String,
    pub options: ResourceGraphQueryRestOptions,
}
