use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::ops::Deref;
use std::str::FromStr;

/// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftauthorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct PolicySetDefinitionName {
    inner: CompactString,
}
crate::impl_facet_string_proxy!(PolicySetDefinitionName, value => value.to_string());
impl PolicySetDefinitionName {
    pub fn new(inner: impl Into<CompactString>) -> Self {
        Self {
            inner: inner.into(),
        }
    }
}
impl Slug for PolicySetDefinitionName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        Ok(())
    }
}

impl FromStr for PolicySetDefinitionName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PolicySetDefinitionName::try_new(s)
    }
}

impl std::fmt::Display for PolicySetDefinitionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.inner))
    }
}
impl Deref for PolicySetDefinitionName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for PolicySetDefinitionName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<PolicySetDefinitionName> for CompactString {
    fn from(value: PolicySetDefinitionName) -> Self {
        value.inner.to_compact_string()
    }
}

cloud_terrastodon_registry::register_thing!(PolicySetDefinitionName);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        let name = facet_json::from_str::<PolicySetDefinitionName>("\"set-definition-name\"")?;
        assert_eq!(
            name,
            PolicySetDefinitionName::try_new("set-definition-name")?
        );
        assert_eq!(facet_json::to_string(&name)?, "\"set-definition-name\"");
        Ok(())
    }
}

