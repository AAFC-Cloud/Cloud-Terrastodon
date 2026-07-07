use arbitrary::Arbitrary;
use std::convert::Infallible;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct UnifiedRoleAssignmentId(String);
crate::impl_facet_string_proxy!(UnifiedRoleAssignmentId, value => value.to_string());
impl UnifiedRoleAssignmentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}
impl AsRef<str> for UnifiedRoleAssignmentId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl std::str::FromStr for UnifiedRoleAssignmentId {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}
impl std::fmt::Display for UnifiedRoleAssignmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::ops::Deref for UnifiedRoleAssignmentId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::UnifiedRoleAssignmentId;

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        let id = facet_json::from_str::<UnifiedRoleAssignmentId>("\"role-assignment-id\"")?;
        assert_eq!(id.as_ref(), "role-assignment-id");
        let reparsed =
            facet_json::from_str::<UnifiedRoleAssignmentId>(&facet_json::to_string(&id)?)?;
        assert_eq!(id, reparsed);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(UnifiedRoleAssignmentId);
cloud_terrastodon_registry::register_arbitrary!(UnifiedRoleAssignmentId);
