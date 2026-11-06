use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Key Vault's integer UTC timestamp representation.
/// See [SecretAttributes](https://learn.microsoft.com/en-us/rest/api/keyvault/secrets/get-secrets/get-secrets?view=rest-keyvault-secrets-2025-07-01#secretattributes).
#[derive(Clone, Copy, Debug, Facet)]
#[facet(transparent)]
pub struct UtcDateTimeEpochSecondsProxy(i64);

impl TryFrom<UtcDateTimeEpochSecondsProxy> for DateTime<Utc> {
    type Error = eyre::Report;
    fn try_from(value: UtcDateTimeEpochSecondsProxy) -> Result<Self, Self::Error> {
        DateTime::from_timestamp(value.0, 0).ok_or_else(|| {
            eyre::eyre!("UTC epoch seconds are outside the supported timestamp range")
        })
    }
}
impl From<&DateTime<Utc>> for UtcDateTimeEpochSecondsProxy {
    fn from(value: &DateTime<Utc>) -> Self {
        Self(value.timestamp())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_out_of_range_seconds_and_retains_pre_epoch_dates() {
        assert!(DateTime::<Utc>::try_from(UtcDateTimeEpochSecondsProxy(i64::MAX)).is_err());
        assert_eq!(
            DateTime::<Utc>::try_from(UtcDateTimeEpochSecondsProxy(-1))
                .unwrap()
                .timestamp(),
            -1
        );
    }
}
