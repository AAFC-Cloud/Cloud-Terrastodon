use facet::Facet;

/// One tag pair and the number of resources using it in the selected inventory.
#[derive(Facet)]
pub(crate) struct AzureTagInventoryRow {
    pub key: (String, String),
    pub name: String,
    pub value: String,
    pub resource_count: usize,
}
