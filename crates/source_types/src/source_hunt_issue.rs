use crate::source_path_proxy::SourcePathProxy;
use arbitrary::Arbitrary;
use facet::Facet;
use std::path::PathBuf;

/// A local read failure that leaves the remaining inventory usable but partial.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
pub struct SourceHuntIssue {
    #[facet(proxy = SourcePathProxy)]
    pub path: PathBuf,
    pub message: String,
}

cloud_terrastodon_registry::register_thing!(SourceHuntIssue);
cloud_terrastodon_registry::register_arbitrary!(SourceHuntIssue);
