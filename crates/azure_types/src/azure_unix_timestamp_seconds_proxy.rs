use eyre::WrapErr;
use facet::Facet;
use jiff::Timestamp;

/// Integer UTC timestamp representation used by
/// [Key Vault](https://learn.microsoft.com/en-us/rest/api/keyvault/secrets/get-secrets/get-secrets?view=rest-keyvault-secrets-2025-07-01#secretattributes)
/// and [Azure CLI access tokens](https://learn.microsoft.com/en-us/cli/azure/authenticate-azure-cli-interactively#refresh-tokens).
#[derive(Clone, Copy, Debug, Facet)]
#[facet(transparent)]
pub struct AzureUnixTimestampSecondsProxy(i64);

impl TryFrom<AzureUnixTimestampSecondsProxy> for Timestamp {
    type Error = eyre::Report;

    fn try_from(value: AzureUnixTimestampSecondsProxy) -> Result<Self, Self::Error> {
        Timestamp::from_second(value.0)
            .wrap_err("UTC epoch seconds are outside the supported timestamp range")
    }
}

impl From<&Timestamp> for AzureUnixTimestampSecondsProxy {
    fn from(value: &Timestamp) -> Self {
        // Preserve the floor convention for fractional seconds before the epoch.
        Self(value.as_second() - i64::from(value.subsec_nanosecond() < 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_out_of_range_seconds_and_retains_pre_epoch_dates() -> eyre::Result<()> {
        assert!(Timestamp::try_from(AzureUnixTimestampSecondsProxy(i64::MAX)).is_err());
        assert_eq!(
            Timestamp::try_from(AzureUnixTimestampSecondsProxy(-1))?.as_second(),
            -1
        );
        assert_eq!(
            AzureUnixTimestampSecondsProxy::from(&Timestamp::from_nanosecond(-1)?).0,
            -1
        );
        Ok(())
    }
}
