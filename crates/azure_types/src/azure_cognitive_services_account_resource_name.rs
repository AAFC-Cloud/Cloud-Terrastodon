use crate::slug::Slug;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use eyre::Context;
use eyre::bail;
use std::ops::Deref;
use std::str::FromStr;

/// Length 2-64.
/// Alphanumerics and hyphens.
/// Start and end with alphanumeric.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AzureCognitiveServicesAccountResourceName {
    inner: CompactString,
}

impl Slug for AzureCognitiveServicesAccountResourceName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        validate_cognitive_services_account_resource_name(&inner)?;
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        validate_cognitive_services_account_resource_name(&self.inner)?;
        Ok(())
    }
}

fn validate_cognitive_services_account_resource_name(value: &CompactString) -> eyre::Result<()> {
    validate_cognitive_services_account_resource_name_inner(value)
        .wrap_err_with(|| format!("Invalid Cognitive Services account resource name: {value}"))
        .wrap_err("https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftcognitiveservices")
}

fn validate_cognitive_services_account_resource_name_inner(
    value: &CompactString,
) -> eyre::Result<()> {
    let char_count = value.chars().count();
    if !(2..=64).contains(&char_count) {
        bail!("Cognitive Services account resource name must be between 2 and 64 characters");
    }

    for (index, character) in value.chars().enumerate() {
        if !(character.is_ascii_alphanumeric() || character == '-') {
            bail!(
                "Char {character} at position {index} in {:?} must be alphanumeric or hyphen",
                value
            );
        }
        if index == 0 && !character.is_ascii_alphanumeric() {
            bail!(
                "First char {character} at position {index} in {:?} must be alphanumeric",
                value
            );
        }
        if index == char_count - 1 && !character.is_ascii_alphanumeric() {
            bail!(
                "Last char {character} at position {index} in {:?} must be alphanumeric",
                value
            );
        }
    }

    Ok(())
}

impl FromStr for AzureCognitiveServicesAccountResourceName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

impl TryFrom<&str> for AzureCognitiveServicesAccountResourceName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl std::fmt::Display for AzureCognitiveServicesAccountResourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

impl serde::Serialize for AzureCognitiveServicesAccountResourceName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AzureCognitiveServicesAccountResourceName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}

impl Deref for AzureCognitiveServicesAccountResourceName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFrom<CompactString> for AzureCognitiveServicesAccountResourceName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<AzureCognitiveServicesAccountResourceName> for CompactString {
    fn from(value: AzureCognitiveServicesAccountResourceName) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for AzureCognitiveServicesAccountResourceName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(2..=64)?;
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

        Self::try_new(CompactString::from(name)).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(test)]
mod test {
    use super::AzureCognitiveServicesAccountResourceName;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::Rng;

    #[test]
    fn validation() -> eyre::Result<()> {
        assert!(AzureCognitiveServicesAccountResourceName::try_new("my-openai").is_ok());
        assert!(AzureCognitiveServicesAccountResourceName::try_new("a1").is_ok());
        assert!(AzureCognitiveServicesAccountResourceName::try_new("abc123").is_ok());
        assert!(AzureCognitiveServicesAccountResourceName::try_new("a".repeat(64)).is_ok());

        assert!(AzureCognitiveServicesAccountResourceName::try_new("a").is_err());
        assert!(AzureCognitiveServicesAccountResourceName::try_new("-abc").is_err());
        assert!(AzureCognitiveServicesAccountResourceName::try_new("abc-").is_err());
        assert!(AzureCognitiveServicesAccountResourceName::try_new("abc.def").is_err());
        assert!(AzureCognitiveServicesAccountResourceName::try_new("abc_def").is_err());
        assert!(AzureCognitiveServicesAccountResourceName::try_new("").is_err());
        assert!(AzureCognitiveServicesAccountResourceName::try_new("a".repeat(65)).is_err());
        Ok(())
    }

    #[test]
    fn fuzz() -> eyre::Result<()> {
        for _ in 0..100 {
            let mut raw = [0u8; 64];
            rand::rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = AzureCognitiveServicesAccountResourceName::arbitrary(&mut un)?;
            assert!(name.validate_slug().is_ok());
        }
        Ok(())
    }
}
