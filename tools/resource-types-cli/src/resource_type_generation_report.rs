use eyre::Result;
use eyre::eyre;
use facet::Facet;
use std::path::Path;

/// Result of writing a normalized catalogue or rendering/checking generated source.
#[derive(Debug, Facet)]
pub struct ResourceTypeGenerationReport {
    pub resource_type_count: usize,
    /// UTF-8 path for presentation; conversion is checked before publication.
    pub output: String,
    pub changed: bool,
}

impl ResourceTypeGenerationReport {
    pub fn new(resource_type_count: usize, output: &Path, changed: bool) -> Result<Self> {
        Ok(Self {
            resource_type_count,
            output: output
                .to_str()
                .ok_or_else(|| eyre!("resource-type output path cannot be represented as UTF-8"))?
                .to_owned(),
            changed,
        })
    }
}
