use eyre::Result;
use eyre::eyre;
use std::path::PathBuf;

/// Lossless string wire representation for native artifact paths in CLI reports.
/// The pinned Facet JSON serializer cannot serialize PathBuf scalars directly.
#[derive(Debug, facet::Facet)]
#[facet(transparent)]
pub struct OperationPathsProxy(pub Vec<String>);

impl TryFrom<&Vec<PathBuf>> for OperationPathsProxy {
    type Error = eyre::Report;

    fn try_from(paths: &Vec<PathBuf>) -> Result<Self> {
        paths
            .iter()
            .map(|path| {
                path.to_str().map(str::to_owned).ok_or_else(|| {
                    eyre!("Iconography artifact path is not valid Unicode: {path:?}")
                })
            })
            .collect::<Result<Vec<_>>>()
            .map(Self)
    }
}

impl From<OperationPathsProxy> for Vec<PathBuf> {
    fn from(proxy: OperationPathsProxy) -> Self {
        proxy.0.into_iter().map(PathBuf::from).collect()
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::OperationPathsProxy;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::path::PathBuf;

    #[test]
    fn non_unicode_artifact_paths_are_reported_instead_of_replaced() {
        let paths = vec![PathBuf::from(OsString::from_wide(&[0xd800]))];
        assert!(OperationPathsProxy::try_from(&paths).is_err());
    }
}
