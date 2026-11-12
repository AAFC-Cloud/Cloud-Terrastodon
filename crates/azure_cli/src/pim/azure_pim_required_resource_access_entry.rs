use facet::Facet;
use uuid::Uuid;

/// One declared application permission inspected by PIM setup.
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePimRequiredResourceAccessEntry {
    pub id: Uuid,
    #[facet(rename = "type")]
    pub permission_type: String,
}
