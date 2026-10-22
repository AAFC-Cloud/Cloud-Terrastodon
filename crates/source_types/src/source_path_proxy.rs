use eyre::Result;
use std::path::PathBuf;

/// String wire representation for native paths in source inventory records.
/// The pinned Facet JSON serializer does not handle PathBuf scalars directly.
/// Reject non-Unicode paths rather than silently changing their identity.
#[derive(Debug, facet::Facet)]
#[facet(transparent)]
pub(crate) struct SourcePathProxy(pub String);

impl TryFrom<&PathBuf> for SourcePathProxy {
    type Error = eyre::Report;

    fn try_from(value: &PathBuf) -> Result<Self> {
        Ok(Self(
            value
                .to_str()
                .ok_or_else(|| {
                    eyre::eyre!("Source inventory path is not valid Unicode: {value:?}")
                })?
                .to_owned(),
        ))
    }
}

impl From<SourcePathProxy> for PathBuf {
    fn from(value: SourcePathProxy) -> Self {
        Self::from(value.0)
    }
}
