use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use std::ops::Deref;
use std::str::FromStr;

/// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftauthorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct RoleManagementPolicyAssignmentName {
    inner: CompactString,
}
crate::impl_facet_string_proxy!(RoleManagementPolicyAssignmentName, value => value.to_string());
impl RoleManagementPolicyAssignmentName {
    pub fn new(inner: CompactString) -> Self {
        Self { inner }
    }
}
impl Slug for RoleManagementPolicyAssignmentName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        Ok(())
    }
}

impl FromStr for RoleManagementPolicyAssignmentName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RoleManagementPolicyAssignmentName::try_new(s)
    }
}

impl std::fmt::Display for RoleManagementPolicyAssignmentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl Deref for RoleManagementPolicyAssignmentName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for RoleManagementPolicyAssignmentName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<RoleManagementPolicyAssignmentName> for CompactString {
    fn from(value: RoleManagementPolicyAssignmentName) -> Self {
        value.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        let name =
            facet_json::from_str::<RoleManagementPolicyAssignmentName>("\"assignment-name\"")?;
        assert_eq!(
            name,
            RoleManagementPolicyAssignmentName::try_new("assignment-name")?
        );
        assert_eq!(facet_json::to_string(&name)?, "\"assignment-name\"");
        Ok(())
    }
}
