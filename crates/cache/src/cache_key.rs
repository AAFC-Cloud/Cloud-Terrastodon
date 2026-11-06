use crate::CacheScope;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use std::path::Path;
use std::time::Duration;

/// A logical scope and a caller's maximum acceptable age for reusable results.
///
/// Validity does not participate in operation ownership. Different age policies
/// for one scope must still agree on its canonical operation and output type.
/// `Duration::ZERO` requests no result reuse; it does not exempt ownership checks.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Facet, Arbitrary)]
pub struct CacheKey {
    pub scope: CacheScope,
    pub valid_for: Duration,
}

impl CacheKey {
    /// Build an unlimited-age key from an explicit relative hierarchy.
    pub fn from_relative_path(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            scope: CacheScope::try_from_relative_path(path)?,
            valid_for: Duration::MAX,
        })
    }

    /// Append a relative hierarchy while preserving this key's freshness policy.
    pub fn join(&self, path: impl AsRef<Path>) -> Result<Self> {
        let suffix = CacheScope::try_from_relative_path(path)?;
        Ok(Self {
            scope: CacheScope::try_from_components(
                self.scope
                    .components()
                    .iter()
                    .chain(suffix.components())
                    .cloned(),
            )?,
            valid_for: self.valid_for,
        })
    }
}

cloud_terrastodon_registry::register_thing!(CacheKey);
cloud_terrastodon_registry::register_arbitrary!(CacheKey);
