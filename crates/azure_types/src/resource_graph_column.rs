use facet::Facet;

/// A column name and reported kind used to reconstruct table rows as objects.
///
/// This describes a column within the table form of
/// [Resource Graph query data](https://learn.microsoft.com/en-us/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01#queryresponse).
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
pub struct ResourceGraphColumn {
    pub name: String,
    #[facet(rename = "type")]
    pub kind: String,
}
