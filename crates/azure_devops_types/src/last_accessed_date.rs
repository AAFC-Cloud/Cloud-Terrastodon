use arbitrary::Arbitrary;
use facet::Facet;
use jiff::Timestamp;
use jiff::tz::Offset;
use std::str::FromStr;

/// The last access timestamp for an [Azure DevOps 7.1 user entitlement](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/get?view=azure-devops-rest-7.1#examples).
#[derive(Debug, Eq, PartialEq, Clone, Facet, Arbitrary)]
#[facet(proxy = String)]
#[repr(C)]
pub enum LastAccessedDate {
    Some(Timestamp),
    Never,
}

impl FromStr for LastAccessedDate {
    type Err = jiff::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let accessed_at = value.parse::<Timestamp>()?;
        let utc = Offset::UTC.to_datetime(accessed_at);
        // The Get and Add examples report year-1 sentinels at different hours.
        if utc.year() == 1 && utc.day_of_year() == 1 {
            Ok(Self::Never)
        } else {
            Ok(Self::Some(accessed_at))
        }
    }
}

impl From<&LastAccessedDate> for String {
    fn from(value: &LastAccessedDate) -> Self {
        match value {
            LastAccessedDate::Some(date) => date.to_string(),
            LastAccessedDate::Never => "0001-01-01T00:00:00+00:00".to_owned(),
        }
    }
}

impl TryFrom<String> for LastAccessedDate {
    type Error = jiff::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_documented_never_accessed_dates() -> eyre::Result<()> {
        for timestamp in [
            "0001-01-01T00:00:00Z",
            "0001-01-01T00:00:00+00:00",
            "0001-01-01T08:00:00Z",
            "0001-01-02T00:30:00+01:00",
        ] {
            assert_eq!(
                timestamp.parse::<LastAccessedDate>()?,
                LastAccessedDate::Never
            );
        }
        Ok(())
    }

    #[test]
    fn preserves_last_access_timestamp() -> eyre::Result<()> {
        let timestamp = "2023-10-05T12:34:56+00:00";
        let accessed = timestamp.parse::<LastAccessedDate>()?;
        assert_eq!(
            accessed,
            LastAccessedDate::Some(timestamp.parse::<Timestamp>()?)
        );
        assert_eq!(
            String::from(&accessed).parse::<LastAccessedDate>()?,
            accessed
        );
        Ok(())
    }
}
