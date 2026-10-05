use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::str::FromStr;

/// A positive maximum number of builds to return.
///
/// Microsoft's [Builds List `$top` parameter](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1#uri-parameters)
/// is an int32. This value is at most `i32::MAX` and rejects zero so a requested
/// list limit has a useful positive meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = u32)]
pub struct AzureDevOpsBuildListLimit(u32);

impl AzureDevOpsBuildListLimit {
    pub fn new(value: u32) -> Result<Self> {
        ensure!(
            value > 0 && value <= i32::MAX as u32,
            "Build list limits must be between 1 and {}",
            i32::MAX
        );
        Ok(Self(value))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for AzureDevOpsBuildListLimit {
    type Error = eyre::Report;

    fn try_from(value: u32) -> Result<Self> {
        Self::new(value)
    }
}

impl From<&AzureDevOpsBuildListLimit> for u32 {
    fn from(value: &AzureDevOpsBuildListLimit) -> Self {
        value.0
    }
}

impl FromStr for AzureDevOpsBuildListLimit {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value.parse()?)
    }
}

impl std::fmt::Display for AzureDevOpsBuildListLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildListLimit {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(u.int_in_range(1..=i32::MAX as u32)?))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildListLimit);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildListLimit);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_list_limits_require_positive_int32_values() {
        for value in [1, 100, i32::MAX as u32] {
            let limit = AzureDevOpsBuildListLimit::new(value).unwrap();
            assert_eq!(limit.get(), value);
            assert_eq!(
                value
                    .to_string()
                    .parse::<AzureDevOpsBuildListLimit>()
                    .unwrap(),
                limit
            );
        }
        for value in [0, i32::MAX as u32 + 1, u32::MAX] {
            assert!(AzureDevOpsBuildListLimit::new(value).is_err());
            assert!(
                value
                    .to_string()
                    .parse::<AzureDevOpsBuildListLimit>()
                    .is_err()
            );
        }
        assert!("-1".parse::<AzureDevOpsBuildListLimit>().is_err());
    }
}
