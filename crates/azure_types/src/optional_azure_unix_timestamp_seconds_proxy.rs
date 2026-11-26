use crate::AzureUnixTimestampSecondsProxy;
use facet::Facet;
use jiff::Timestamp;

/// Optional Key Vault UTC timestamps retain absence without inventing a date.
#[derive(Clone, Copy, Debug, Facet)]
#[facet(transparent)]
pub struct OptionalAzureUnixTimestampSecondsProxy(Option<AzureUnixTimestampSecondsProxy>);

impl TryFrom<OptionalAzureUnixTimestampSecondsProxy> for Option<Timestamp> {
    type Error = eyre::Report;
    fn try_from(value: OptionalAzureUnixTimestampSecondsProxy) -> Result<Self, Self::Error> {
        value.0.map(Timestamp::try_from).transpose()
    }
}
impl From<&Option<Timestamp>> for OptionalAzureUnixTimestampSecondsProxy {
    fn from(value: &Option<Timestamp>) -> Self {
        Self(value.as_ref().map(AzureUnixTimestampSecondsProxy::from))
    }
}
