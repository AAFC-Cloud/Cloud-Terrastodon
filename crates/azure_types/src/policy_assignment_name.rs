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
pub struct PolicyAssignmentName {
    pub inner: CompactString,
}
impl PolicyAssignmentName {
    pub fn new(inner: CompactString) -> Self {
        Self { inner }
    }
}
impl Slug for PolicyAssignmentName {
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
impl serde::Serialize for PolicyAssignmentName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for PolicyAssignmentName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| D::Error::custom(format!("{e:?}")))
    }
}
impl Deref for PolicyAssignmentName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for PolicyAssignmentName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
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
