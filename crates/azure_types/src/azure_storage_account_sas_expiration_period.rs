use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use eyre::eyre;
use facet::Facet;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

/// SAS lifetime encoded as days and a clock component (`DD.HH:MM:SS`).
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#saspolicy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Facet)]
#[facet(proxy = String)]
pub struct AzureStorageAccountSasExpirationPeriod {
    // Whole seconds preserve the documented wire precision without floating point.
    seconds: u64,
}

impl AzureStorageAccountSasExpirationPeriod {
    pub const fn from_seconds(seconds: u64) -> Self {
        Self { seconds }
    }

    pub const fn duration(&self) -> Duration {
        Duration::from_secs(self.seconds)
    }
}

impl FromStr for AzureStorageAccountSasExpirationPeriod {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        let (days, time) = value
            .split_once('.')
            .ok_or_else(|| eyre!("SAS expiration period must use DD.HH:MM:SS"))?;
        let components = time.split(':').collect::<Vec<_>>();
        let [hours, minutes, seconds] = components.as_slice() else {
            return Err(eyre!("SAS expiration period must use DD.HH:MM:SS"));
        };
        let mut parsed = [0u64; 4];
        for (destination, component) in parsed.iter_mut().zip([days, *hours, *minutes, *seconds]) {
            ensure!(
                !component.is_empty() && component.bytes().all(|byte| byte.is_ascii_digit()),
                "SAS expiration period components must be decimal numbers"
            );
            *destination = component
                .parse()
                .wrap_err("SAS expiration period component is too large")?;
        }
        let [days, hours, minutes, seconds] = parsed;
        ensure!(
            hours < 24 && minutes < 60 && seconds < 60,
            "SAS expiration period clock components are outside their ranges"
        );
        let seconds = days
            .checked_mul(86_400)
            .and_then(|days| days.checked_add(hours * 3_600 + minutes * 60 + seconds))
            .ok_or_else(|| eyre!("SAS expiration period exceeds the supported duration"))?;
        Ok(Self { seconds })
    }
}

impl fmt::Display for AzureStorageAccountSasExpirationPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{:02}:{:02}:{:02}",
            self.seconds / 86_400,
            self.seconds / 3_600 % 24,
            self.seconds / 60 % 60,
            self.seconds % 60
        )
    }
}

impl<'a> Arbitrary<'a> for AzureStorageAccountSasExpirationPeriod {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::from_seconds(u.arbitrary()?))
    }
}

crate::impl_facet_string_proxy!(AzureStorageAccountSasExpirationPeriod, value => value.to_string());
cloud_terrastodon_registry::register_thing!(AzureStorageAccountSasExpirationPeriod);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountSasExpirationPeriod);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expiration_period_preserves_duration_and_rejects_ambiguous_or_overflowing_clocks()
    -> Result<()> {
        let period: AzureStorageAccountSasExpirationPeriod = "1.02:03:04".parse()?;
        assert_eq!(period.duration().as_secs(), 93_784);
        assert_eq!(period.to_string(), "1.02:03:04");
        for invalid in [
            "1.24:00:00",
            "1.00:60:00",
            "1.00:00:60",
            "-1.01:00:00",
            "01:00:00",
            "1.00:00",
            "18446744073709551615.00:00:00",
        ] {
            assert!(
                invalid
                    .parse::<AzureStorageAccountSasExpirationPeriod>()
                    .is_err(),
                "{invalid}"
            );
        }
        let maximum = AzureStorageAccountSasExpirationPeriod::from_seconds(u64::MAX);
        assert_eq!(
            maximum
                .to_string()
                .parse::<AzureStorageAccountSasExpirationPeriod>()?,
            maximum
        );
        Ok(())
    }
}
