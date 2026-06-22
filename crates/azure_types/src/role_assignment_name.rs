use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

/// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftauthorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct RoleAssignmentName {
    inner: Uuid,
}
crate::impl_facet_string_proxy!(RoleAssignmentName, value => value.to_string());
impl RoleAssignmentName {
    pub fn new(inner: Uuid) -> Self {
        Self { inner }
    }
}
impl Slug for RoleAssignmentName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner: Uuid = name.into().parse()?;
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        // UUID is always valid
        Ok(())
    }
}

impl FromStr for RoleAssignmentName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RoleAssignmentName::try_new(s)
    }
}

impl std::fmt::Display for RoleAssignmentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.inner.hyphenated()))
    }
}
impl Deref for RoleAssignmentName {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for RoleAssignmentName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<RoleAssignmentName> for CompactString {
    fn from(value: RoleAssignmentName) -> Self {
        value.inner.as_hyphenated().to_compact_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        let json = "\"00000000-0000-0000-0000-000000000000\"";
        let name = facet_json::from_str::<RoleAssignmentName>(json)?;
        assert_eq!(name, RoleAssignmentName::new(Uuid::nil()));
        assert_eq!(facet_json::to_string(&name)?, json);
        Ok(())
    }
}
