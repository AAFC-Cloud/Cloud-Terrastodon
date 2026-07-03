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
pub struct RoleDefinitionName {
    inner: Uuid,
}
crate::impl_facet_string_proxy!(RoleDefinitionName, value => value.to_string());
impl RoleDefinitionName {
    pub fn new(inner: Uuid) -> Self {
        Self { inner }
    }
}
impl Slug for RoleDefinitionName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner: Uuid = name.into().parse()?;
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        // UUID is always valid
        Ok(())
    }
}

impl FromStr for RoleDefinitionName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RoleDefinitionName::try_new(s)
    }
}

impl std::fmt::Display for RoleDefinitionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.inner.hyphenated()))
    }
}
impl Deref for RoleDefinitionName {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for RoleDefinitionName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<RoleDefinitionName> for CompactString {
    fn from(value: RoleDefinitionName) -> Self {
        value.inner.as_hyphenated().to_compact_string()
    }
}

cloud_terrastodon_registry::register_thing!(RoleDefinitionName);
cloud_terrastodon_registry::register_arbitrary!(RoleDefinitionName);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        let json = "\"00000000-0000-0000-0000-000000000000\"";
        let name = facet_json::from_str::<RoleDefinitionName>(json)?;
        assert_eq!(name, RoleDefinitionName::new(Uuid::nil()));
        assert_eq!(facet_json::to_string(&name)?, json);
        Ok(())
    }
}
