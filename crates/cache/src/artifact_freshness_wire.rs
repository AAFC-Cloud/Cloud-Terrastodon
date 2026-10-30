use crate::ArtifactFreshness;
use crate::artifact_expiry::ArtifactExpiry;
use crate::artifact_timestamp_proxy::ArtifactTimestampProxy;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Wire boundary that requires an observation and explicit expiry policy.
///
/// Both fields must be present when decoding; legacy or damaged records must
/// not acquire unbounded freshness from an omitted optional expiry field.
/// Typed timestamps use a field proxy to preserve nanoseconds on the wire.
#[derive(Clone, Debug, Facet)]
pub(crate) struct ArtifactFreshnessWire {
    #[facet(proxy = ArtifactTimestampProxy)]
    pub observed_at: DateTime<Utc>,
    pub expires_at: ArtifactExpiry,
}

impl From<&ArtifactFreshness> for ArtifactFreshnessWire {
    fn from(freshness: &ArtifactFreshness) -> Self {
        Self {
            observed_at: freshness.observed_at,
            expires_at: match freshness.expires_at {
                Some(expiry) => ArtifactExpiry::At(expiry),
                None => ArtifactExpiry::Unbounded,
            },
        }
    }
}
