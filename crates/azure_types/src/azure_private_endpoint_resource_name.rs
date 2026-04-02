use crate::slug::Slug;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use eyre::Context;
use eyre::bail;
use std::ops::Deref;
use std::str::FromStr;

/// Length 2-64.
/// Alphanumerics, underscores, periods, and hyphens.
/// Start with alphanumeric. End with alphanumeric or underscore.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AzurePrivateEndpointResourceName {
    inner: CompactString,
}

impl Slug for AzurePrivateEndpointResourceName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        validate_private_endpoint_resource_name(&inner)?;
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        validate_private_endpoint_resource_name(&self.inner)?;
        Ok(())
    }
}

fn validate_private_endpoint_resource_name(value: &CompactString) -> eyre::Result<()> {
    validate_private_endpoint_resource_name_inner(value)
        .wrap_err_with(|| format!("Invalid private endpoint resource name: {}", value))
        .wrap_err("https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftnetwork")
}

fn validate_private_endpoint_resource_name_inner(value: &CompactString) -> eyre::Result<()> {
    let char_count = value.chars().count();
    if !(2..=64).contains(&char_count) {
        bail!("Private endpoint resource name must be between 2 and 64 characters");
    }
    for (i, char) in value.chars().enumerate() {
        if !(char.is_ascii_alphanumeric() || char == '-' || char == '_' || char == '.') {
            bail!(
                "Char {} at position {} in {:?} must be alphanumeric, hyphen, underscore, or period",
                char,
                i,
                value
            );
        }
        if i == 0 && !char.is_ascii_alphanumeric() {
            bail!(
                "First char {} at position {} in {:?} must be alphanumeric",
                char,
                i,
                value
            );
        }
        if i == char_count - 1 && !(char.is_ascii_alphanumeric() || char == '_') {
            bail!(
                "Last char {} at position {} in {:?} must be alphanumeric or underscore",
                char,
                i,
                value
            );
        }
    }
    Ok(())
}

impl FromStr for AzurePrivateEndpointResourceName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AzurePrivateEndpointResourceName::try_new(s)
    }
}

impl TryFrom<&str> for AzurePrivateEndpointResourceName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        AzurePrivateEndpointResourceName::try_new(value)
    }
}

impl std::fmt::Display for AzurePrivateEndpointResourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

impl serde::Serialize for AzurePrivateEndpointResourceName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AzurePrivateEndpointResourceName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}

impl Deref for AzurePrivateEndpointResourceName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFrom<CompactString> for AzurePrivateEndpointResourceName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<AzurePrivateEndpointResourceName> for CompactString {
    fn from(value: AzurePrivateEndpointResourceName) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for AzurePrivateEndpointResourceName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(2..=64)?;

        let edge_choices = ('a'..='z')
            .chain('A'..='Z')
            .chain('0'..='9')
            .collect::<Vec<_>>();
        let middle_choices = edge_choices
            .iter()
            .copied()
            .chain(['-', '_', '.'])
            .collect::<Vec<_>>();
        let last_choices = edge_choices
            .iter()
            .copied()
            .chain(['_'])
            .collect::<Vec<_>>();

        let first = *u.choose(&edge_choices)?;
        let mut name = String::with_capacity(len);
        name.push(first);

        for _ in 0..(len - 2) {
            name.push(*u.choose(&middle_choices)?);
        }
        name.push(*u.choose(&last_choices)?);

        AzurePrivateEndpointResourceName::try_new(CompactString::from(name))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(test)]
mod test {
    use crate::AzurePrivateEndpointResourceName;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::Rng;

    #[test]
    pub fn validation() -> eyre::Result<()> {
        assert!(AzurePrivateEndpointResourceName::try_new("pe01").is_ok());
        assert!(AzurePrivateEndpointResourceName::try_new("ab").is_ok());
        assert!(AzurePrivateEndpointResourceName::try_new("a-b-c").is_ok());
        assert!(AzurePrivateEndpointResourceName::try_new("abc.def").is_ok());
        assert!(AzurePrivateEndpointResourceName::try_new("abc_def").is_ok());
        assert!(AzurePrivateEndpointResourceName::try_new("a".repeat(64)).is_ok());

        assert!(AzurePrivateEndpointResourceName::try_new("a").is_err());
        assert!(AzurePrivateEndpointResourceName::try_new("-a").is_err());
        assert!(AzurePrivateEndpointResourceName::try_new("hi+hi").is_err());
        assert!(AzurePrivateEndpointResourceName::try_new("").is_err());
        assert!(AzurePrivateEndpointResourceName::try_new(".abc").is_err());
        assert!(AzurePrivateEndpointResourceName::try_new("_abc").is_err());
        assert!(AzurePrivateEndpointResourceName::try_new("abc-").is_err());
        assert!(AzurePrivateEndpointResourceName::try_new("abc.").is_err());
        assert!(AzurePrivateEndpointResourceName::try_new("a".repeat(65)).is_err());
        Ok(())
    }

    #[test]
    pub fn fuzz() -> eyre::Result<()> {
        for _ in 0..100 {
            let mut raw = [0u8; 64];
            rand::rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = AzurePrivateEndpointResourceName::arbitrary(&mut un)?;
            assert!(name.validate_slug().is_ok());
        }
        Ok(())
    }
}
