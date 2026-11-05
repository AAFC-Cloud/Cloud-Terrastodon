use crate::UtcDateTimeEpochSecondsProxy;
use chrono::DateTime;
use chrono::Utc;

/// Optional Key Vault UTC timestamps retain absence without inventing a date.
#[derive(Clone, Copy, Debug, facet::Facet)]
#[facet(transparent)]
pub struct OptionalUtcDateTimeEpochSecondsProxy(Option<UtcDateTimeEpochSecondsProxy>);

impl TryFrom<OptionalUtcDateTimeEpochSecondsProxy> for Option<DateTime<Utc>> {
    type Error = eyre::Report;
    fn try_from(value: OptionalUtcDateTimeEpochSecondsProxy) -> Result<Self, Self::Error> {
        value.0.map(DateTime::try_from).transpose()
    }
}
impl From<&Option<DateTime<Utc>>> for OptionalUtcDateTimeEpochSecondsProxy {
    fn from(value: &Option<DateTime<Utc>>) -> Self {
        Self(value.as_ref().map(UtcDateTimeEpochSecondsProxy::from))
    }
}
