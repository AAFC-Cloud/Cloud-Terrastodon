use crate::slug::Slug;
use compact_str::CompactString;
use eyre::bail;
use std::hash::Hash;
use std::ops::Deref;
use std::str::FromStr;

/// Represents the name component of a service group resource id.
///
/// Can be up to [250 characters](https://learn.microsoft.com/en-us/azure/governance/service-groups/overview#important-facts-about-service-groups)
#[derive(Debug, Clone, Eq, PartialOrd, Ord)]
pub struct ServiceGroupName {
    inner: CompactString,
}
impl PartialEq for ServiceGroupName {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq_ignore_ascii_case(&other.inner)
    }
}
impl Hash for ServiceGroupName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.to_ascii_lowercase().hash(state);
    }
}

impl Slug for ServiceGroupName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        validate_service_group_name(&inner)?;
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        validate_service_group_name(&self.inner)
    }
}

impl FromStr for ServiceGroupName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ServiceGroupName::try_new(s)
    }
}
impl TryFrom<String> for ServiceGroupName {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ServiceGroupName::try_new(value)
    }
}
impl TryFrom<&str> for ServiceGroupName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ServiceGroupName::try_new(value)
    }
}
impl TryFrom<&String> for ServiceGroupName {
    type Error = eyre::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        ServiceGroupName::try_new(value)
    }
}

impl std::fmt::Display for ServiceGroupName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl serde::Serialize for ServiceGroupName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}
impl<'de> serde::Deserialize<'de> for ServiceGroupName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        ServiceGroupName::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}
impl Deref for ServiceGroupName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl From<ServiceGroupName> for CompactString {
    fn from(value: ServiceGroupName) -> Self {
        value.inner
    }
}

fn validate_service_group_name(value: &str) -> eyre::Result<()> {
    if value.is_empty() {
        bail!("Service group name cannot be empty");
    }
    if value.len() > 250 {
        bail!("Service group name cannot be longer than 250 characters");
    }
    for (idx, ch) in value.chars().enumerate() {
        if matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '(' | ')' | '.' | '~') {
            continue;
        }
        bail!(
            "Service group name contains invalid character '{}' at position {}",
            ch,
            idx
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation() {
        assert!(ServiceGroupName::try_new("My-Service_Group~1").is_ok());
        assert!(ServiceGroupName::try_new("").is_err());
        assert!(ServiceGroupName::try_new("has space").is_err());
        assert!(ServiceGroupName::try_new("unicode-π").is_err());
    }
}
