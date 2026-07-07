use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use eyre::bail;
use std::hash::Hash;
use std::ops::Deref;
use std::str::FromStr;

/// Represents the name component of a service group resource id.
///
/// Can be up to [250 characters](https://learn.microsoft.com/en-us/azure/governance/service-groups/overview#important-facts-about-service-groups)
#[derive(Debug, Clone, Eq, PartialOrd, Ord, facet::Facet)]
#[facet(json::proxy = String)]
pub struct ServiceGroupName {
    inner: CompactString,
}
crate::impl_facet_string_proxy_serialize!(ServiceGroupName, value => value.to_string());
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

impl<'a> Arbitrary<'a> for ServiceGroupName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(1..=64)?;
        let mut value = String::with_capacity(len);
        for _ in 0..len {
            let idx = u.int_in_range(0..=67)?;
            value.push(match idx {
                0..=25 => (b'a' + idx as u8) as char,
                26..=51 => (b'A' + (idx - 26) as u8) as char,
                52..=61 => (b'0' + (idx - 52) as u8) as char,
                62 => '-',
                63 => '_',
                64 => '(',
                65 => ')',
                66 => '.',
                _ => '~',
            });
        }
        ServiceGroupName::try_new(value).map_err(|_| arbitrary::Error::IncorrectFormat)
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

cloud_terrastodon_registry::register_thing!(ServiceGroupName);
cloud_terrastodon_registry::register_arbitrary!(ServiceGroupName);

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

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        let name = facet_json::from_str::<ServiceGroupName>("\"My-Service_Group~1\"")?;
        assert_eq!(name, ServiceGroupName::try_new("My-Service_Group~1")?);
        assert_eq!(facet_json::to_string(&name)?, "\"My-Service_Group~1\"");
        Ok(())
    }
}
