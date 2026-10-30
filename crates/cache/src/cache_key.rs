use crate::CacheScope;
use arbitrary::Arbitrary;
use facet::Facet;
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

cloud_terrastodon_registry_values::register_thing!(CacheKey);
cloud_terrastodon_registry_values::register_arbitrary!(CacheKey);
