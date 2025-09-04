use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::ops::Deref;
use tracing::warn;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RolePermissionAction {
    inner: String,
}

impl Serialize for RolePermissionAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.inner)
    }
}

impl<'de> Deserialize<'de> for RolePermissionAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = RolePermissionAction { inner: expanded };
        Ok(id)
    }
}
impl Deref for RolePermissionAction {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RolePermissionAction {
    pub fn new(inner: impl Into<String>) -> Self {
        Self {
            inner: inner.into(),
        }
    }
    pub fn satisfies(&self, other: &RolePermissionAction) -> bool {
        other.is_satisfied_by(self)
    }
    pub fn is_satisfied_by(&self, other: &RolePermissionAction) -> bool {
        if self.inner == other.inner {
            return true;
        }
        if self.contains('*') {
            warn!("Checking if action with wildcard satisfies another action is not supported");
        }
        if other.inner.contains('*') {
            // action needs:
            // Microsoft.KeyVault/vaults/secrets/readMetadata/action
            // user has
            // Microsoft.KeyVault/vaults/secrets/*/action

            // must begin with the part before the start and end with the part after?
            let parts: Vec<&str> = other.inner.split('*').collect();
            if parts.len() == 2 {
                return self.inner.starts_with(parts[0]) && self.inner.ends_with(parts[1]);
            }
        }
        return false;
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let a = super::RolePermissionAction::new("Microsoft.KeyVault/vaults/secrets/readMetadata/action");
        let b = super::RolePermissionAction::new("Microsoft.KeyVault/vaults/secrets/*/action");
        assert!(a.is_satisfied_by(&b));
        Ok(())
    }
    #[test]
    pub fn exact_match_is_satisfied() -> eyre::Result<()> {
        let a = super::RolePermissionAction::new("Microsoft.Storage/accounts/read/action");
        let b = super::RolePermissionAction::new("Microsoft.Storage/accounts/read/action");
        assert!(a.is_satisfied_by(&b));
        Ok(())
    }

    #[test]
    pub fn wildcard_at_end_is_satisfied() -> eyre::Result<()> {
        let a = super::RolePermissionAction::new("Microsoft.Storage/accounts/read/action");
        let b = super::RolePermissionAction::new("Microsoft.Storage/accounts/*");
        assert!(a.is_satisfied_by(&b));
        Ok(())
    }

    #[test]
    pub fn wildcard_at_start_is_satisfied() -> eyre::Result<()> {
        let a = super::RolePermissionAction::new("Microsoft.Storage/accounts/read/action");
        let b = super::RolePermissionAction::new("*/read/action");
        assert!(a.is_satisfied_by(&b));
        Ok(())
    }

    #[test]
    pub fn wildcard_in_middle_is_satisfied() -> eyre::Result<()> {
        let a = super::RolePermissionAction::new("Microsoft.Storage/accounts/read/action");
        let b = super::RolePermissionAction::new("Microsoft.Storage/*/action");
        assert!(a.is_satisfied_by(&b));
        Ok(())
    }

    #[test]
    pub fn wildcard_multiple_parts_is_not_satisfied() -> eyre::Result<()> {
        let a = super::RolePermissionAction::new("Microsoft.Storage/accounts/read/action");
        let b = super::RolePermissionAction::new("Microsoft.*/*/action");
        assert!(!a.is_satisfied_by(&b));
        Ok(())
    }

    #[test]
    pub fn no_wildcard_and_not_equal_is_not_satisfied() -> eyre::Result<()> {
        let a = super::RolePermissionAction::new("Microsoft.Storage/accounts/write/action");
        let b = super::RolePermissionAction::new("Microsoft.Storage/accounts/read/action");
        assert!(!a.is_satisfied_by(&b));
        Ok(())
    }
}