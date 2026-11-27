use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fmt;
use std::str::FromStr;

/// Login name used with the [Helix AR System REST API](https://docs.helixops.ai/bin/Service-Management/Innovation-Suite/BMC-Helix-Innovation-Suite/is254/Developing-an-API-program/Integrating-AR-System-forms-with-a-third-party-application-by-using-the-REST-API/).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[facet(proxy = String)]
pub struct BmcHelixUsername(String);

impl BmcHelixUsername {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for BmcHelixUsername {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_from(value.to_owned())
    }
}

impl TryFrom<String> for BmcHelixUsername {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        ensure!(
            !value.trim().is_empty() && !value.chars().any(char::is_control),
            "Helix username must be nonempty and contain no control characters"
        );
        Ok(Self(value))
    }
}

impl From<&BmcHelixUsername> for String {
    fn from(value: &BmcHelixUsername) -> Self {
        value.0.clone()
    }
}

impl fmt::Display for BmcHelixUsername {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'a> Arbitrary<'a> for BmcHelixUsername {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(format!("test-user-{}", u32::arbitrary(input)?)))
    }
}

cloud_terrastodon_registry::register_thing!(BmcHelixUsername);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixUsername);
