use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::ops::Deref;
use std::str::FromStr;

/// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftauthorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct PolicyAssignmentName {
    inner: CompactString,
}
crate::impl_facet_string_proxy!(PolicyAssignmentName, value => value.to_string());
impl PolicyAssignmentName {
    pub fn new(inner: CompactString) -> Self {
        Self { inner }
    }
}
impl Slug for PolicyAssignmentName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        Ok(())
    }
}

impl FromStr for PolicyAssignmentName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PolicyAssignmentName::try_new(s)
    }
}

impl std::fmt::Display for PolicyAssignmentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.inner))
    }
}
impl Deref for PolicyAssignmentName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for PolicyAssignmentName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<PolicyAssignmentName> for CompactString {
    fn from(value: PolicyAssignmentName) -> Self {
        value.inner.to_compact_string()
    }
}

cloud_terrastodon_registry::register_thing!(PolicyAssignmentName);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        let name = facet_json::from_str::<PolicyAssignmentName>("\"assignment-name\"")?;
        assert_eq!(name, PolicyAssignmentName::try_new("assignment-name")?);
        assert_eq!(facet_json::to_string(&name)?, "\"assignment-name\"");
        Ok(())
    }
}
