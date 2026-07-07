// TODO: fork iso8601_duration crate to support Facet and Arbitrary features, replace `nom` dependency with manual parsing or `winnow`

use arbitrary::Arbitrary;
use facet::Facet;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Facet)]
#[facet(default, opaque, proxy = IsoDurationProxy)]
pub struct IsoDuration(iso8601_duration::Duration);
impl Deref for IsoDuration {
    type Target = iso8601_duration::Duration;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<iso8601_duration::Duration> for IsoDuration {
    fn from(value: iso8601_duration::Duration) -> Self {
        Self(value)
    }
}
impl From<IsoDuration> for iso8601_duration::Duration {
    fn from(value: IsoDuration) -> Self {
        value.0
    }
}
impl Default for IsoDuration {
    fn default() -> Self {
        Self(iso8601_duration::Duration::parse("PT0S").unwrap())
    }
}
impl Arbitrary<'_> for IsoDuration {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        // Generate a simple ISO8601 duration string like "PT{n}S" where n is a non-negative integer
        let secs: u64 = u.arbitrary()?;
        let s = format!("PT{}S", secs % 1_000_000);
        // Parse; fall back to zero seconds on any parse error
        match iso8601_duration::Duration::parse(&s) {
            Ok(d) => Ok(IsoDuration(d)),
            Err(_) => Ok(IsoDuration(
                iso8601_duration::Duration::parse("PT0S").unwrap(),
            )),
        }
    }
}
impl From<std::time::Duration> for IsoDuration {
    fn from(value: std::time::Duration) -> Self {
        let total_seconds = value.as_secs();
        let seconds = (total_seconds % 60) as f32;
        let total_minutes = total_seconds / 60;
        let minutes = (total_minutes % 60) as f32;
        let total_hours = total_minutes / 60;
        let hours = (total_hours % 24) as f32;
        let days = (total_hours / 24) as f32;

        IsoDuration(iso8601_duration::Duration::new(
            0.0, 0.0, days, hours, minutes, seconds,
        ))
    }
}
impl FromStr for IsoDuration {
    type Err = iso8601_duration::ParseDurationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        iso8601_duration::Duration::parse(s).map(IsoDuration)
    }
}

#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(transparent)]
pub struct IsoDurationProxy(String);

impl TryFrom<IsoDurationProxy> for IsoDuration {
    type Error = String;

    fn try_from(value: IsoDurationProxy) -> Result<Self, Self::Error> {
        value.0.parse().map_err(|err| format!("{err:?}"))
    }
}

impl From<&IsoDuration> for IsoDurationProxy {
    fn from(value: &IsoDuration) -> Self {
        Self(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_iso8601() {
        let duration = std::time::Duration::new(28800, 0); // 8 hours in seconds
        let iso_string = IsoDuration::from(duration).to_string();
        assert_eq!(iso_string, "PT8H");

        let duration = std::time::Duration::new(3661, 0); // 1 hour, 1 minute, and 1 second
        let iso_string = IsoDuration::from(duration).to_string();
        assert_eq!(iso_string, "PT1H1M1S");

        let duration = std::time::Duration::new(90061, 0); // 1 day, 1 hour, 1 minute, and 1 second
        let iso_string = IsoDuration::from(duration).to_string();
        assert_eq!(iso_string, "P1DT1H1M1S");
    }
}
