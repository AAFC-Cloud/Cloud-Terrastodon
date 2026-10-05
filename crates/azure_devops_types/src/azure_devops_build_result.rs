use arbitrary::Arbitrary;
use std::str::FromStr;

/// A build result used in API records and build-list filters.
///
/// Microsoft's [API vocabulary](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1#buildresult) is recognized exactly.
/// Unknown values retain their original wire spelling for forward compatibility.
/// Filters use the same API spelling, including any unknown value supplied by
/// the caller; Azure DevOps determines which filter values it accepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsBuildResult {
    None,
    Succeeded,
    PartiallySucceeded,
    Failed,
    Canceled,
    Unknown(String),
}

impl AzureDevOpsBuildResult {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Succeeded => "succeeded",
            Self::PartiallySucceeded => "partiallySucceeded",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
            Self::Unknown(value) => value,
        }
    }
}

impl FromStr for AzureDevOpsBuildResult {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "none" => Self::None,
            "succeeded" => Self::Succeeded,
            "partiallySucceeded" => Self::PartiallySucceeded,
            "failed" => Self::Failed,
            "canceled" => Self::Canceled,
            value => Self::Unknown(value.to_owned()),
        })
    }
}

impl TryFrom<String> for AzureDevOpsBuildResult {
    type Error = std::convert::Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsBuildResult> for String {
    fn from(value: &AzureDevOpsBuildResult) -> Self {
        value.as_str().to_owned()
    }
}

impl std::fmt::Display for AzureDevOpsBuildResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildResult);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildResult);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_result_preserves_known_and_unknown_wire_values() {
        for (wire, expected) in [
            ("none", AzureDevOpsBuildResult::None),
            ("succeeded", AzureDevOpsBuildResult::Succeeded),
            (
                "partiallySucceeded",
                AzureDevOpsBuildResult::PartiallySucceeded,
            ),
            ("failed", AzureDevOpsBuildResult::Failed),
            ("canceled", AzureDevOpsBuildResult::Canceled),
            (
                "futureSyntheticValue",
                AzureDevOpsBuildResult::Unknown("futureSyntheticValue".to_owned()),
            ),
            ("NONE", AzureDevOpsBuildResult::Unknown("NONE".to_owned())),
        ] {
            let value: AzureDevOpsBuildResult = wire.parse().unwrap();
            assert_eq!(value, expected);
            assert_eq!(value.as_str(), wire);
        }
    }
}
