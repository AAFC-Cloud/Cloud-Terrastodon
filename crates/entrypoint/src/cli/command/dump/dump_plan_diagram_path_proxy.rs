use eyre::Result;
use std::path::PathBuf;

/// Optional native path represented as a JSON string, preserving its identity.
/// The pinned Facet JSON serializer does not support PathBuf scalars directly.
#[derive(Debug, facet::Facet)]
#[facet(transparent)]
pub(super) struct DumpPlanDiagramPathProxy(pub Option<String>);

impl TryFrom<&Option<PathBuf>> for DumpPlanDiagramPathProxy {
    type Error = eyre::Report;

    fn try_from(value: &Option<PathBuf>) -> Result<Self> {
        Ok(Self(
            value
                .as_ref()
                .map(|path| {
                    path.to_str()
                        .map(str::to_owned)
                        .ok_or_else(|| eyre::eyre!("Diagram path is not valid Unicode: {path:?}"))
                })
                .transpose()?,
        ))
    }
}

impl From<DumpPlanDiagramPathProxy> for Option<PathBuf> {
    fn from(value: DumpPlanDiagramPathProxy) -> Self {
        value.0.map(PathBuf::from)
    }
}
