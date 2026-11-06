use facet::Facet;
use std::collections::BTreeMap;

/// Semantic transport inputs, excluding acquired credentials and presentation.
#[derive(Facet)]
pub(crate) struct RestRequestFingerprint {
    pub(crate) service: String,
    pub(crate) method: String,
    pub(crate) url: String,
    pub(crate) body: Option<String>,
    pub(crate) headers: BTreeMap<String, Vec<String>>,
}
