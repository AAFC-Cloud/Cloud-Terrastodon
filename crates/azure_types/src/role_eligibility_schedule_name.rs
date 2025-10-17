use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

/// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftauthorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Arbitrary)]
pub struct RoleEligibilityScheduleName {
    inner: Uuid,
}
impl RoleEligibilityScheduleName {
    pub fn new(inner: Uuid) -> Self {
        Self { inner }
    }
}
impl Slug for RoleEligibilityScheduleName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner: Uuid = name.into().parse()?;
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        // UUID is always valid
        Ok(())
    }
}

impl FromStr for RoleEligibilityScheduleName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RoleEligibilityScheduleName::try_new(s)
    }
}

impl std::fmt::Display for RoleEligibilityScheduleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.inner.hyphenated()))
    }
}
impl serde::Serialize for RoleEligibilityScheduleName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for RoleEligibilityScheduleName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}
impl Deref for RoleEligibilityScheduleName {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for RoleEligibilityScheduleName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<RoleEligibilityScheduleName> for CompactString {
    fn from(value: RoleEligibilityScheduleName) -> Self {
        value.inner.as_hyphenated().to_compact_string()
    }
}
