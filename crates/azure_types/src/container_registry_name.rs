use crate::slug::Slug;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use eyre::Context;
use eyre::bail;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContainerRegistryName {
    inner: CompactString,
}
impl Slug for ContainerRegistryName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        validate_container_registry_name(&inner)?;
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        validate_container_registry_name(&self.inner)?;
        Ok(())
    }
}
fn validate_container_registry_name(value: &CompactString) -> eyre::Result<()> {
    validate_container_registry_name_inner(value)
        .wrap_err_with(|| format!("Invalid container registry name: {}", value))
        .wrap_err("https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftcontainerregistry")
}

fn validate_container_registry_name_inner(value: &CompactString) -> eyre::Result<()> {
    let char_count = value.chars().count();
    if !(5..=50).contains(&char_count) {
        bail!("Container registry name must be between 5 and 50 characters");
    }
    for (i, char) in value.chars().enumerate() {
        if !char.is_ascii_alphanumeric() {
            bail!(
                "Char {} at position {} in {:?} must be alphanumeric",
                char,
                i,
                value
            );
        }
    }
    Ok(())
}
impl<'a> Arbitrary<'a> for ContainerRegistryName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Get length in 3-24
        let len = u.int_in_range(5..=50)?;
        // Use only [a-z]
        let choices = ('a'..='z')
            .chain('A'..='Z')
            .chain('1'..='9')
            .collect::<Vec<_>>();
        let name: String = (0..len)
            .map(|_| Ok(*u.choose(&choices)?))
            .collect::<Result<String, _>>()?;
        ContainerRegistryName::try_new(CompactString::from(name))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

impl FromStr for ContainerRegistryName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ContainerRegistryName::try_new(s)
    }
}
impl TryFrom<&str> for ContainerRegistryName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ContainerRegistryName::try_new(value)
    }
}
impl std::fmt::Display for ContainerRegistryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl serde::Serialize for ContainerRegistryName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ContainerRegistryName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}
impl Deref for ContainerRegistryName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for ContainerRegistryName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<ContainerRegistryName> for CompactString {
    fn from(value: ContainerRegistryName) -> Self {
        value.inner
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::ContainerRegistryName;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::Rng;

    #[test]
    pub fn validation() -> eyre::Result<()> {
        assert!(ContainerRegistryName::try_new("bruhs").is_ok());
        assert!(ContainerRegistryName::try_new("-").is_err());
        assert!(ContainerRegistryName::try_new("a-b-c").is_err());
        assert!(ContainerRegistryName::try_new("hi+hi").is_err());
        assert!(ContainerRegistryName::try_new("").is_err());
        assert!(ContainerRegistryName::try_new("a").is_err());
        assert!(ContainerRegistryName::try_new("aa").is_err());
        assert!(ContainerRegistryName::try_new("JEOFF").is_ok());
        assert!(ContainerRegistryName::try_new("caPital").is_ok());
        assert!(ContainerRegistryName::try_new("aaaa").is_err());
        assert!(ContainerRegistryName::try_new("a".repeat(23)).is_ok());
        assert!(ContainerRegistryName::try_new("a".repeat(49)).is_ok());
        assert!(ContainerRegistryName::try_new("a".repeat(55)).is_err());
        Ok(())
    }

    #[test]
    #[ignore]
    pub fn preview_failure() -> eyre::Result<()> {
        ContainerRegistryName::try_new("abc123---B321")?;
        Ok(())
    }

    #[test]
    pub fn fuzz() -> eyre::Result<()> {
        let mut found_uppercase = false;
        for _ in 0..100 {
            let mut raw = [0u8; 64];
            rand::rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = ContainerRegistryName::arbitrary(&mut un)?;
            assert!(name.validate_slug().is_ok());
            if name.chars().any(|c| c.is_ascii_uppercase()) {
                found_uppercase = true;
            }
            println!("{name}");
        }
        assert!(
            found_uppercase,
            "At least one generated name should contain an uppercase character"
        );
        Ok(())
    }
}
