use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

/// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftauthorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PartialOrd, Ord, Arbitrary)]
pub struct RoleDefinitionName {
    inner: Uuid,
}
impl RoleDefinitionName {
    pub fn new(inner: Uuid) -> Self {
        Self { inner }
    }
}
impl Slug for RoleDefinitionName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let rtn = Self {
            inner: name.into().parse()?,
        };
        rtn.validate()?;
        Ok(rtn)
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        self.validate()?;
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
impl serde::Serialize for RoleDefinitionName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for RoleDefinitionName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
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
