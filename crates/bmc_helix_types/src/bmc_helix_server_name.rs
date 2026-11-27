use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fmt;
use std::str::FromStr;

/// Name of a saved Helix server in the local Cloud Terrastodon catalogue.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[facet(proxy = String)]
pub struct BmcHelixServerName(String);

impl BmcHelixServerName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for BmcHelixServerName {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_from(value.to_owned())
    }
}

impl TryFrom<String> for BmcHelixServerName {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        ensure!(
            !value.is_empty()
                && !value
                    .chars()
                    .any(|character| character.is_whitespace() || character.is_control()),
            "Helix server name must be nonempty and contain no whitespace or control characters"
        );
        Ok(Self(value))
    }
}

impl From<&BmcHelixServerName> for String {
    fn from(value: &BmcHelixServerName) -> Self {
        value.0.clone()
    }
}

impl fmt::Display for BmcHelixServerName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'a> Arbitrary<'a> for BmcHelixServerName {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(format!("helix-{}", u32::arbitrary(input)?)))
    }
}

cloud_terrastodon_registry::register_thing!(BmcHelixServerName);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixServerName);
