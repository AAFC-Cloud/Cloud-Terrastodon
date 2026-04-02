use crate::slug::Slug;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use eyre::Context;
use eyre::bail;
use std::ops::Deref;
use std::str::FromStr;

/// Length 2-60.
/// Alphanumeric or hyphen characters, including Unicode alphanumerics.
/// Cannot start or end with a hyphen.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AzureAppServiceResourceName {
    inner: CompactString,
}

impl Slug for AzureAppServiceResourceName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        validate_app_service_resource_name(&inner)?;
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        validate_app_service_resource_name(&self.inner)?;
        Ok(())
    }
}

fn validate_app_service_resource_name(value: &CompactString) -> eyre::Result<()> {
    validate_app_service_resource_name_inner(value)
        .wrap_err_with(|| format!("Invalid app service resource name: {}", value))
        .wrap_err("https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftweb")
}

fn validate_app_service_resource_name_inner(value: &CompactString) -> eyre::Result<()> {
    let char_count = value.chars().count();
    if !(2..=60).contains(&char_count) {
        bail!("App service resource name must be between 2 and 60 characters");
    }

    for (index, character) in value.chars().enumerate() {
        if !(character.is_alphanumeric() || character == '-') {
            bail!(
                "Char {} at position {} in {:?} must be alphanumeric or hyphen",
                character,
                index,
                value
            );
        }

        if (index == 0 || index == char_count - 1) && character == '-' {
            bail!("App service resource name cannot start or end with a hyphen");
        }
    }

    Ok(())
}

impl FromStr for AzureAppServiceResourceName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AzureAppServiceResourceName::try_new(s)
    }
}

impl TryFrom<&str> for AzureAppServiceResourceName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        AzureAppServiceResourceName::try_new(value)
    }
}

impl std::fmt::Display for AzureAppServiceResourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

impl serde::Serialize for AzureAppServiceResourceName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AzureAppServiceResourceName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}

impl Deref for AzureAppServiceResourceName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFrom<CompactString> for AzureAppServiceResourceName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<AzureAppServiceResourceName> for CompactString {
    fn from(value: AzureAppServiceResourceName) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for AzureAppServiceResourceName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(2..=60)?;
        let edge_choices = ('a'..='z')
            .chain('A'..='Z')
            .chain('0'..='9')
            .collect::<Vec<_>>();
        let middle_choices = edge_choices
            .iter()
            .copied()
            .chain(['-'])
            .collect::<Vec<_>>();

        let first = *u.choose(&edge_choices)?;
        let last = *u.choose(&edge_choices)?;
        let mut name = String::with_capacity(len);
        name.push(first);

        for _ in 0..(len - 2) {
            name.push(*u.choose(&middle_choices)?);
        }

        name.push(last);

        AzureAppServiceResourceName::try_new(CompactString::from(name))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(test)]
mod test {
    use crate::AzureAppServiceResourceName;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::Rng;

    #[test]
    fn validation() -> eyre::Result<()> {
        assert!(AzureAppServiceResourceName::try_new("my-app-service").is_ok());
        assert!(AzureAppServiceResourceName::try_new("ab").is_ok());
        assert!(AzureAppServiceResourceName::try_new("appservice01").is_ok());
        assert!(AzureAppServiceResourceName::try_new("cafeservice").is_ok());
        assert!(AzureAppServiceResourceName::try_new("a".repeat(60)).is_ok());

        assert!(AzureAppServiceResourceName::try_new("a").is_err());
        assert!(AzureAppServiceResourceName::try_new("-app").is_err());
        assert!(AzureAppServiceResourceName::try_new("app-").is_err());
        assert!(AzureAppServiceResourceName::try_new("app_service").is_err());
        assert!(AzureAppServiceResourceName::try_new("app service").is_err());
        assert!(AzureAppServiceResourceName::try_new("a".repeat(61)).is_err());
        Ok(())
    }

    #[test]
    fn fuzz() -> eyre::Result<()> {
        for _ in 0..100 {
            let mut raw = [0u8; 64];
            rand::rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = AzureAppServiceResourceName::arbitrary(&mut un)?;
            assert!(name.validate_slug().is_ok());
        }
        Ok(())
    }
}
