use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use eyre::bail;
use std::ops::Deref;
use std::str::FromStr;

// TODO: update to conform more closely to <https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftcompute>
// Simple VM name validation: 1-64 characters, allow alphanumeric, '-', '_' and '.'
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VirtualMachineName {
    inner: CompactString,
}

impl Slug for VirtualMachineName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        validate_vm_name(&inner)?;
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        validate_vm_name(&self.inner)?;
        Ok(())
    }
}

fn validate_vm_name(value: &str) -> eyre::Result<()> {
    let len = value.chars().count();
    if !(1..=64).contains(&len) {
        bail!(
            "Virtual machine name must be between 1 and 64 characters, got {}",
            len
        );
    }

    let chars: Vec<char> = value.chars().collect();
    // Must start with alphanumeric
    let first_char = chars
        .first()
        .ok_or_else(|| eyre::eyre!("Virtual machine name cannot be empty"))?;
    if !first_char.is_alphanumeric() {
        bail!("Virtual machine name must start with alphanumeric character");
    }

    // Must end with alphanumeric or underscore
    let last_char = chars.last().unwrap();
    if !(last_char.is_alphanumeric() || *last_char == '_' || *last_char == '-') {
        bail!("Virtual machine name must end with alphanumeric character, underscore or hyphen");
    }

    for (i, c) in chars.iter().enumerate() {
        if !(c.is_alphanumeric() || *c == '_' || *c == '.' || *c == '-') {
            bail!(
                "Virtual machine name contains invalid character '{}' at position {}",
                c,
                i
            );
        }
    }

    Ok(())
}

impl FromStr for VirtualMachineName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

impl TryFrom<&str> for VirtualMachineName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl std::fmt::Display for VirtualMachineName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl serde::Serialize for VirtualMachineName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for VirtualMachineName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = CompactString::deserialize(deserializer)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

impl Deref for VirtualMachineName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFrom<CompactString> for VirtualMachineName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<VirtualMachineName> for CompactString {
    fn from(value: VirtualMachineName) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for VirtualMachineName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(1..=64)?;
        let first_char = if u.ratio(1, 2)? {
            if u.ratio(1, 2)? {
                u.int_in_range(b'a'..=b'z')? as char
            } else {
                u.int_in_range(b'A'..=b'Z')? as char
            }
        } else {
            u.int_in_range(b'0'..=b'9')? as char
        };
        let mut name = String::with_capacity(len);
        name.push(first_char);
        for _ in 1..len.saturating_sub(1) {
            let ch = match u.int_in_range(0..=4)? {
                0 => u.int_in_range(b'a'..=b'z')? as char,
                1 => u.int_in_range(b'A'..=b'Z')? as char,
                2 => u.int_in_range(b'0'..=b'9')? as char,
                3 => '_',
                _ => '-',
            };
            name.push(ch);
        }
        if len > 1 {
            let last = if u.ratio(3, 4)? {
                u.int_in_range(b'0'..=b'9')? as char
            } else {
                '_'
            };
            name.push(last);
        }
        VirtualMachineName::try_new(name).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names() -> eyre::Result<()> {
        assert!(VirtualMachineName::try_new("my-vm").is_ok());
        assert!(VirtualMachineName::try_new("a").is_ok());
        assert!(VirtualMachineName::try_new(&"a".repeat(64)).is_ok());
        Ok(())
    }

    #[test]
    fn invalid_names() {
        assert!(VirtualMachineName::try_new("!").is_err());
    }
}
