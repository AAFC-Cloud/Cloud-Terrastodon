use chrono::DateTime;
use chrono::SecondsFormat;
use chrono::Utc;
use eyre::Context;
use eyre::Report;
use eyre::Result;
use facet::Facet;

/// Lossless RFC3339 wire conversion for artifact timestamps.
///
/// The pinned Facet implementation formats `DateTime<Utc>` to whole seconds.
/// Keep timestamps typed in their owning records and preserve nanoseconds at
/// this serialization boundary. Invalid wire values fail before reaching them.
#[derive(Debug, Facet)]
#[facet(transparent)]
pub(crate) struct ArtifactTimestampProxy(String);

impl From<&DateTime<Utc>> for ArtifactTimestampProxy {
    fn from(timestamp: &DateTime<Utc>) -> Self {
        Self(timestamp.to_rfc3339_opts(SecondsFormat::Nanos, true))
    }
}

impl TryFrom<ArtifactTimestampProxy> for DateTime<Utc> {
    type Error = Report;

    fn try_from(timestamp: ArtifactTimestampProxy) -> Result<Self> {
        DateTime::parse_from_rfc3339(&timestamp.0)
            .map(|parsed| parsed.with_timezone(&Utc))
            .wrap_err("artifact timestamp is not an RFC3339 timestamp")
    }
}
