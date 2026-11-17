use crate::IconFamily;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;
use url::Url;

/// One selected asset, its public identity, and the provenance of its exact bytes.
#[derive(Clone, Debug, Facet)]
#[facet(deny_unknown_fields)]
pub struct IconCatalogueEntry {
    pub family: IconFamily,
    pub variant: String,
    pub name: String,
    pub category: String,
    pub file: String,
    pub source_url: Url,
    pub logical_name: String,
    pub discovery_url: Url,
    pub sha256: String,
    pub retrieved_at: DateTime<Utc>,
    pub terms_url: Url,
    pub resource_types: Vec<String>,
}
