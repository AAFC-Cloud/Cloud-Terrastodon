use arbitrary::Arbitrary;
use compact_str::CompactString;
use compact_str::ToCompactString;
use serde::de::Error;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;
use validator::Validate;

use crate::slug::Slug;

/// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftauthorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PartialOrd, Ord, Arbitrary)]
pub struct PolicyDefinitionName {
    pub inner: CompactString,
}
impl PolicyDefinitionName {
    pub fn new(inner: CompactString) -> Self {
        Self {
            inner
        }
    }
}
impl Slug for PolicyDefinitionName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let rtn = Self { inner: name.into().parse()? };
        rtn.validate()?;
        Ok(rtn)
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        self.validate()?;
        Ok(())
    }
}

impl FromStr for PolicyDefinitionName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PolicyDefinitionName::try_new(s)
    }
}

impl std::fmt::Display for PolicyDefinitionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.inner))
    }
}
impl serde::Serialize for PolicyDefinitionName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for PolicyDefinitionName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| D::Error::custom(format!("{e:?}")))
    }
}
impl Deref for PolicyDefinitionName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for PolicyDefinitionName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl TryFrom<CompactString> for PolicyDefinitionName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<PolicyDefinitionName> for CompactString {
    fn from(value: PolicyDefinitionName) -> Self {
        value.inner.to_compact_string()
    }
}
