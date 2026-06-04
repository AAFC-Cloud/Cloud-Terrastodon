use compact_str::CompactString;
use eyre::bail;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GiteaOwnerName {
    inner: CompactString,
}

impl GiteaOwnerName {
    pub fn try_new(value: impl Into<CompactString>) -> eyre::Result<Self> {
        let value = value.into();
        validate_segment("owner", &value)?;
        Ok(Self { inner: value })
    }
}

pub(crate) fn validate_segment(kind: &str, value: &str) -> eyre::Result<()> {
    let value = value.trim();
    if value.is_empty() {
        bail!("{kind} cannot be empty");
    }
    if value.contains('/') {
        bail!("{kind} cannot contain '/'");
    }
    Ok(())
}

impl Display for GiteaOwnerName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

impl Deref for GiteaOwnerName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<str> for GiteaOwnerName {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl FromStr for GiteaOwnerName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

impl Serialize for GiteaOwnerName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GiteaOwnerName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = <CompactString as Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}
