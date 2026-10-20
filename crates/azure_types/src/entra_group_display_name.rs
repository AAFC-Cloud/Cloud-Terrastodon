use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Report;
use eyre::Result;
use eyre::ensure;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Deref;
use std::str::FromStr;

/// An Entra group's display name, preserving its spelling and whitespace.
///
/// Microsoft Graph's [group `displayName` property](https://learn.microsoft.com/en-us/graph/api/resources/group?view=graph-rest-1.0#properties)
/// is required on creation, cannot be cleared during updates, and allows at most
/// 256 characters. Construction and string deserialization enforce a nonempty
/// value of at most 256 Unicode characters without adding character restrictions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct EntraGroupDisplayName(String);

impl EntraGroupDisplayName {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let length = value.chars().count();
        ensure!(
            (1..=256).contains(&length),
            "Entra group display name must contain 1 to 256 characters, got {length}"
        );
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for EntraGroupDisplayName {
    type Err = Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for EntraGroupDisplayName {
    type Error = Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for EntraGroupDisplayName {
    type Error = Report;

    fn try_from(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<EntraGroupDisplayName> for String {
    fn from(value: EntraGroupDisplayName) -> Self {
        value.0
    }
}

impl From<&EntraGroupDisplayName> for String {
    fn from(value: &EntraGroupDisplayName) -> Self {
        value.0.clone()
    }
}

impl Display for EntraGroupDisplayName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl AsRef<str> for EntraGroupDisplayName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for EntraGroupDisplayName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<'a> Arbitrary<'a> for EntraGroupDisplayName {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let length = input.int_in_range(1..=256)?;
        let name = (0..length)
            .map(|_| char::arbitrary(input))
            .collect::<arbitrary::Result<String>>()?;
        Self::try_new(name).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupDisplayName);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupDisplayName);

#[cfg(test)]
mod tests {
    use super::EntraGroupDisplayName;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;

    #[test]
    fn validates_character_count_without_normalizing_names() {
        let spelling = "  Équipe / Operations  ";
        let name = EntraGroupDisplayName::try_new(spelling).unwrap();
        assert_eq!(name.as_str(), spelling);
        assert!(EntraGroupDisplayName::try_new("é".repeat(256)).is_ok());
        assert!(EntraGroupDisplayName::try_new("é".repeat(257)).is_err());
        assert!(EntraGroupDisplayName::try_new("").is_err());
    }

    #[test]
    fn string_wire_decoding_cannot_bypass_validation() {
        for invalid in [String::new(), "é".repeat(257)] {
            let json = facet_json::to_string(&invalid).unwrap();
            assert!(facet_json::from_str::<EntraGroupDisplayName>(&json).is_err());
        }
        let name = EntraGroupDisplayName::try_new("Équipe / Operations").unwrap();
        assert_eq!(
            facet_json::to_string(&name).unwrap(),
            facet_json::to_string(name.as_str()).unwrap()
        );
    }

    #[test]
    fn arbitrary_names_satisfy_the_same_invariants() {
        for seed in 0..=u8::MAX {
            let bytes = [seed; 4096];
            let name = EntraGroupDisplayName::arbitrary(&mut Unstructured::new(&bytes)).unwrap();
            assert!(EntraGroupDisplayName::try_new(name.as_str()).is_ok());
        }
    }
}
