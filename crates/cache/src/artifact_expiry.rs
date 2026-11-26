use crate::artifact_timestamp_proxy::ArtifactTimestampProxy;
use facet::Facet;
use jiff::Timestamp;

/// Required persisted validity: an explicit deadline or explicitly unbounded.
///
/// Unlike `Option<Timestamp>`, this representation cannot infer unbounded
/// validity when the expiry field is absent. It intentionally has no default.
#[derive(Clone, Debug, Facet)]
#[repr(u8)]
pub(crate) enum ArtifactExpiry {
    /// An exact deadline, serialized as RFC3339 with nanosecond precision.
    At(#[facet(proxy = ArtifactTimestampProxy)] Timestamp),
    Unbounded,
}
