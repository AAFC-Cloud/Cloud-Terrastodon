use std::convert::Infallible;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UnifiedRoleAssignmentId(String);
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
impl<'de> serde::Deserialize<'de> for UnifiedRoleAssignmentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}
impl serde::Serialize for UnifiedRoleAssignmentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}
