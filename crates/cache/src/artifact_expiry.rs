use crate::artifact_timestamp_proxy::ArtifactTimestampProxy;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Required persisted validity: an explicit deadline or explicitly unbounded.
///
/// Unlike `Option<DateTime<Utc>>`, this representation cannot infer unbounded
/// validity when the expiry field is absent. It intentionally has no default.
#[derive(Clone, Debug, Facet)]
#[repr(u8)]
pub(crate) enum ArtifactExpiry {
    /// An exact deadline, serialized as RFC3339 with nanosecond precision.
    At(#[facet(proxy = ArtifactTimestampProxy)] DateTime<Utc>),
    Unbounded,
}
