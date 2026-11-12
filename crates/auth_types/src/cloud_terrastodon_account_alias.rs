use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fmt;
use std::str::FromStr;

/// Human account selector, unique only within its tenant or service namespace.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Facet)]
#[facet(json::proxy = String)]
pub struct CloudTerrastodonAccountAlias(String);

impl CloudTerrastodonAccountAlias {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value.eq_ignore_ascii_case("default"),
            "account alias 'default' is reserved; omit --account to select the configured default"
        );
        ensure!(!value.trim().is_empty(), "account alias must not be empty");
        ensure!(
            value.trim() == value && !value.chars().any(char::is_control),
            "account alias must not contain surrounding whitespace or control characters"
        );
        ensure!(
            value.parse::<crate::CloudTerrastodonAccountId>().is_err(),
            "account aliases cannot be UUIDs because UUID selectors identify accounts directly"
        );
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl FromStr for CloudTerrastodonAccountAlias {
    type Err = eyre::Report;
    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}
impl TryFrom<String> for CloudTerrastodonAccountAlias {
    type Error = eyre::Report;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}
impl From<&CloudTerrastodonAccountAlias> for String {
    fn from(value: &CloudTerrastodonAccountAlias) -> Self {
        value.0.clone()
    }
}
impl fmt::Display for CloudTerrastodonAccountAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl<'a> Arbitrary<'a> for CloudTerrastodonAccountAlias {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(format!("account-{}", u.arbitrary::<u64>()?)))
    }
}
cloud_terrastodon_registry::register_thing!(CloudTerrastodonAccountAlias);
cloud_terrastodon_registry::register_arbitrary!(CloudTerrastodonAccountAlias);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_is_reserved_at_every_alias_construction_boundary() -> Result<()> {
        for alias in ["default", "Default", "DEFAULT"] {
            assert!(CloudTerrastodonAccountAlias::new(alias).is_err());
            assert!(alias.parse::<CloudTerrastodonAccountAlias>().is_err());
            assert!(
                facet_json::from_str::<CloudTerrastodonAccountAlias>(&format!("\"{alias}\""))
                    .is_err()
            );
        }
        assert_eq!(
            "admin".parse::<CloudTerrastodonAccountAlias>()?.as_str(),
            "admin"
        );
        Ok(())
    }
}
