use eyre::Context;
use eyre::Report;
use eyre::Result;
use facet::Facet;
use jiff::Timestamp;

/// Lossless RFC3339 wire conversion for artifact timestamps.
///
/// Keep the existing fixed nine-digit precision in persisted artifacts.
#[derive(Debug, Facet)]
#[facet(transparent)]
pub(crate) struct ArtifactTimestampProxy(String);

impl From<&Timestamp> for ArtifactTimestampProxy {
    fn from(timestamp: &Timestamp) -> Self {
        Self(format!("{timestamp:.9}"))
    }
}

impl TryFrom<ArtifactTimestampProxy> for Timestamp {
    type Error = Report;

    fn try_from(timestamp: ArtifactTimestampProxy) -> Result<Self> {
        timestamp
            .0
            .parse()
            .wrap_err("artifact timestamp is not an RFC3339 timestamp")
    }
}
