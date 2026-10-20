use crate::DesireDocument;
use cloud_terrastodon::azure_types::AzureTenantId;
use eyre::Report;
use eyre::Result;
use std::path::Path;

/// The locally validated document's scope, without authentication or directory reads.
#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ValidationReport {
    pub group_count: usize,
    pub tenant_ids: Vec<AzureTenantId>,
    pub source_digest: String,
}

impl ValidationReport {
    /// Read and validate desired state locally, before any authentication preflight.
    pub fn try_from_path(path: &Path) -> Result<Self> {
        let desire = DesireDocument::try_from_path(path)?;
        let mut tenant_ids: Vec<_> = desire
            .groups()
            .iter()
            .map(|group| group.tenant_id)
            .collect();
        tenant_ids.sort_by_key(ToString::to_string);
        tenant_ids.dedup();
        Ok(Self {
            group_count: desire.groups().len(),
            tenant_ids,
            source_digest: desire.source_digest().to_owned(),
        })
    }
}

impl TryFrom<&Path> for ValidationReport {
    type Error = Report;

    fn try_from(path: &Path) -> Result<Self> {
        Self::try_from_path(path)
    }
}
