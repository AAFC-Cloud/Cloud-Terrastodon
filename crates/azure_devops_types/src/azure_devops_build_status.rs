use arbitrary::Arbitrary;
use std::str::FromStr;

/// A build status used in API records and build-list filters.
///
/// Microsoft's [API vocabulary](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1#buildstatus) is recognized exactly.
/// Unknown values retain their original wire spelling for forward compatibility.
/// Filters use the same API spelling, including any unknown value supplied by
/// the caller; Azure DevOps determines which filter values it accepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsBuildStatus {
    None,
    InProgress,
    Completed,
    Cancelling,
    Postponed,
    NotStarted,
    All,
    Unknown(String),
}

impl AzureDevOpsBuildStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::InProgress => "inProgress",
            Self::Completed => "completed",
            Self::Cancelling => "cancelling",
            Self::Postponed => "postponed",
            Self::NotStarted => "notStarted",
            Self::All => "all",
            Self::Unknown(value) => value,
        }
    }
}

impl FromStr for AzureDevOpsBuildStatus {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "none" => Self::None,
            "inProgress" => Self::InProgress,
            "completed" => Self::Completed,
            "cancelling" => Self::Cancelling,
            "postponed" => Self::Postponed,
            "notStarted" => Self::NotStarted,
            "all" => Self::All,
            value => Self::Unknown(value.to_owned()),
        })
    }
}

impl TryFrom<String> for AzureDevOpsBuildStatus {
    type Error = std::convert::Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsBuildStatus> for String {
    fn from(value: &AzureDevOpsBuildStatus) -> Self {
        value.as_str().to_owned()
    }
}

impl std::fmt::Display for AzureDevOpsBuildStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildStatus);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildStatus);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_status_preserves_known_and_unknown_wire_values() {
        for (wire, expected) in [
            ("none", AzureDevOpsBuildStatus::None),
            ("inProgress", AzureDevOpsBuildStatus::InProgress),
            ("completed", AzureDevOpsBuildStatus::Completed),
            ("cancelling", AzureDevOpsBuildStatus::Cancelling),
            ("postponed", AzureDevOpsBuildStatus::Postponed),
            ("notStarted", AzureDevOpsBuildStatus::NotStarted),
            ("all", AzureDevOpsBuildStatus::All),
            (
                "futureSyntheticValue",
                AzureDevOpsBuildStatus::Unknown("futureSyntheticValue".to_owned()),
            ),
            ("NONE", AzureDevOpsBuildStatus::Unknown("NONE".to_owned())),
        ] {
            let value: AzureDevOpsBuildStatus = wire.parse().unwrap();
            assert_eq!(value, expected);
            assert_eq!(value.as_str(), wire);
        }
    }
}
