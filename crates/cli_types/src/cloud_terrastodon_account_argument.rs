use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use facet::Facet;
use std::fmt;
use std::str::FromStr;

/// Explicit account selection by stable identity or an alias in the command's scope.
/// Parsing does not resolve defaults, configuration, credentials, or service scope.
#[derive(Debug, Clone, Eq, PartialEq, Facet, Arbitrary)]
#[facet(proxy = String)]
#[repr(u8)]
pub enum CloudTerrastodonAccountArgument {
    Id(CloudTerrastodonAccountId),
    Alias(CloudTerrastodonAccountAlias),
}

impl FromStr for CloudTerrastodonAccountArgument {
    type Err = eyre::Report;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = value.parse() {
            Ok(Self::Id(id))
        } else {
            value.parse().map(Self::Alias)
        }
    }
}
impl TryFrom<String> for CloudTerrastodonAccountArgument {
    type Error = eyre::Report;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl From<&CloudTerrastodonAccountArgument> for String {
    fn from(value: &CloudTerrastodonAccountArgument) -> Self {
        value.to_string()
    }
}
impl fmt::Display for CloudTerrastodonAccountArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(value) => value.fmt(f),
            Self::Alias(value) => value.fmt(f),
        }
    }
}
cloud_terrastodon_registry::register_thing!(CloudTerrastodonAccountArgument);
cloud_terrastodon_registry::register_arbitrary!(CloudTerrastodonAccountArgument);
