use facet::Facet;
/// Table output for [Resource Graph queries](https://learn.microsoft.com/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01#resultformat).
#[derive(Debug, Facet)]
#[repr(C)]
pub enum QueryRestResultFormat {
    #[facet(rename = "table")]
    Table,
}
