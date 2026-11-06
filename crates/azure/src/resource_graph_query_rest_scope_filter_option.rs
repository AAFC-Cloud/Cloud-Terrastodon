/// Scope selection for [Resource Graph queries](https://learn.microsoft.com/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01#authorizationscopefilter).
#[derive(Debug, facet::Facet)]
#[repr(C)]
pub enum ResourceGraphQueryRestScopeFilterOption {
    AtScopeAboveAndBelow,
}
