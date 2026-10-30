use eyre::Report;
use eyre::Result;
use eyre::ensure;
use facet::Facet;

/// An owned string containing at least one character.
///
/// Whitespace, case, and Unicode are preserved exactly. Nonempty does not imply
/// nonblank, a particular fingerprint format, or suitability for a file path.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Facet)]
#[facet(proxy = String)]
pub struct NonEmptyString {
    // Private storage prevents mutation from bypassing nonempty validation.
    value: String,
}

impl NonEmptyString {
    /// Reject an empty value without normalizing its contents.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        Self::try_from(value.into())
    }

    /// Inspect the exact validated value without permitting mutation.
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl TryFrom<String> for NonEmptyString {
    type Error = Report;

    fn try_from(value: String) -> Result<Self> {
        ensure!(!value.is_empty(), "string must not be empty");
        Ok(Self { value })
    }
}

impl From<&NonEmptyString> for String {
    fn from(value: &NonEmptyString) -> Self {
        value.value.clone()
    }
}

cloud_terrastodon_registry_values::register_thing!(NonEmptyString);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_and_decoding_reject_empty_values() {
        assert!(NonEmptyString::new("").is_err());
        let encoded = facet_json::to_string(&String::new()).unwrap();
        assert!(facet_json::from_str::<NonEmptyString>(&encoded).is_err());
    }

    #[test]
    fn preserves_nonempty_values_and_their_string_wire_representation() {
        for original in [" ", "\0", " Input/é ", "input/é"] {
            let value = NonEmptyString::new(original).unwrap();
            assert_eq!(value.as_str(), original);
            let encoded = facet_json::to_string(&value).unwrap();
            assert_eq!(encoded, facet_json::to_string(&original).unwrap());
            let decoded = facet_json::from_str::<NonEmptyString>(&encoded).unwrap();
            assert_eq!(decoded, value);
        }
    }
}
