use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Datelike;
use chrono::Utc;
use facet::Facet;
use std::str::FromStr;

/// The last access timestamp for an [Azure DevOps 7.1 user entitlement](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/get?view=azure-devops-rest-7.1#examples).
#[derive(Debug, Eq, PartialEq, Clone, Facet, Arbitrary)]
#[facet(proxy = String)]
#[repr(C)]
pub enum LastAccessedDate {
    Some(DateTime<Utc>),
    Never,
}

impl FromStr for LastAccessedDate {
    type Err = chrono::ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let accessed_at = DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc);
        // The Get and Add examples report year-1 sentinels at different hours.
        if accessed_at.year() == 1 && accessed_at.ordinal() == 1 {
            Ok(Self::Never)
        } else {
            Ok(Self::Some(accessed_at))
        }
    }
}

impl From<&LastAccessedDate> for String {
    fn from(value: &LastAccessedDate) -> Self {
        match value {
            LastAccessedDate::Some(date) => date.to_rfc3339(),
            LastAccessedDate::Never => "0001-01-01T00:00:00+00:00".to_owned(),
        }
    }
}

impl TryFrom<String> for LastAccessedDate {
    type Error = chrono::ParseError;

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
        assert_eq!(
            timestamp.parse::<LastAccessedDate>()?,
            LastAccessedDate::Some(timestamp.parse::<DateTime<Utc>>()?)
        );
        Ok(())
    }
}
