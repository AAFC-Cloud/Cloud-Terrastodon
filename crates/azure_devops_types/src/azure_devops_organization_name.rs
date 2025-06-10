use arbitrary::Arbitrary;
use compact_str::CompactString;
use serde::de::Error;
use std::ops::Deref;
use std::str::FromStr;
use validator::Validate;
use validator::ValidationError;

/// https://learn.microsoft.com/en-us/azure/devops/organizations/settings/naming-restrictions?view=azure-devops#organization-names
///
/// Use only letters from the English alphabet
///
/// Start your organization name with a letter or number
///
/// Use letters, numbers, or hyphens after the initial character
///
/// Ensure that your organization doesn't exceed 50 Unicode characters
///
/// End with a letter or number
#[derive(Debug, Eq, PartialEq, Clone, Validate, Hash)]
pub struct AzureDevOpsOrganizationName {
    #[validate(
        length(min = 1, max = 50),
        custom(function = "validate_azure_devops_organization_name_contents")
    )]
    inner: CompactString,
}

fn is_english_letter(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}

fn is_valid_first_char(ch: char) -> bool {
    is_english_letter(ch) || is_digit(ch)
}

fn is_valid_middle_char(ch: char) -> bool {
    is_english_letter(ch) || is_digit(ch) || ch == '-'
}

fn is_valid_last_char(ch: char) -> bool {
    is_english_letter(ch) || is_digit(ch)
}

fn validate_azure_devops_organization_name_contents(
    value: &CompactString,
) -> Result<(), ValidationError> {
    let s: &str = value;

    if s.is_empty() || s.len() > 50 {
        return Err(ValidationError::new(
            "length must be between 1 and 50 characters",
        ));
    }

    let chars: Vec<char> = s.chars().collect();

    // Check first character
    if let Some(&first_char) = chars.first() {
        if !is_valid_first_char(first_char) {
            return Err(
                ValidationError::new("must start with English letter or number").with_message(
                    format!(
                        "First character '{}' is not an English letter or digit",
                        first_char
                    )
                    .into(),
                ),
            );
        }
    }

    // Check last character (if different from first)
    if chars.len() > 1 {
        if let Some(&last_char) = chars.last() {
            if !is_valid_last_char(last_char) {
                return Err(
                    ValidationError::new("must end with English letter or number").with_message(
                        format!(
                            "Last character '{}' is not an English letter or digit",
                            last_char
                        )
                        .into(),
                    ),
                );
            }
        }
    }

    // Check all characters
    for (i, &ch) in chars.iter().enumerate() {
        if i == 0 {
            // First char already checked
            continue;
        } else if i == chars.len() - 1 && chars.len() > 1 {
            // Last char already checked
            continue;
        } else {
            // Middle characters
            if !is_valid_middle_char(ch) {
                return Err(ValidationError::new("invalid character")
                    .with_message(format!("Character '{}' at position {} is not allowed. Only English letters, digits, and hyphens are allowed", ch, i).into()));
            }
        }
    }

    Ok(())
}

impl Deref for AzureDevOpsOrganizationName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::fmt::Display for AzureDevOpsOrganizationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

impl AzureDevOpsOrganizationName {
    pub fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let org = Self { inner: name.into() };
        org.validate()?;
        Ok(org)
    }
}

impl FromStr for AzureDevOpsOrganizationName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

impl AsRef<str> for AzureDevOpsOrganizationName {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl TryFrom<&str> for AzureDevOpsOrganizationName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsOrganizationName {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

fn rand_valid_first_char(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<char> {
    const VALID_FIRST: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    Ok(*u.choose(VALID_FIRST)? as char)
}

fn rand_valid_middle_char(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<char> {
    const VALID_MIDDLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-";
    Ok(*u.choose(VALID_MIDDLE)? as char)
}

fn rand_valid_last_char(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<char> {
    const VALID_LAST: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    Ok(*u.choose(VALID_LAST)? as char)
}

impl<'a> Arbitrary<'a> for AzureDevOpsOrganizationName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        for _ in 0..40 {
            let len = u.int_in_range(1..=50)?;
            let mut chars = Vec::with_capacity(len);

            if len == 1 {
                chars.push(rand_valid_first_char(u)?);
            } else {
                chars.push(rand_valid_first_char(u)?);
                for _ in 1..len - 1 {
                    chars.push(rand_valid_middle_char(u)?);
                }
                chars.push(rand_valid_last_char(u)?);
            }

            let candidate: String = chars.into_iter().collect();

            if validate_azure_devops_organization_name_contents(&CompactString::from(
                candidate.as_str(),
            ))
            .is_ok()
            {
                return Ok(AzureDevOpsOrganizationName {
                    inner: CompactString::from(candidate),
                });
            }
        }
        Err(arbitrary::Error::IncorrectFormat)
    }
}

impl serde::Serialize for AzureDevOpsOrganizationName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AzureDevOpsOrganizationName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| D::Error::custom(format!("{e:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;

    #[test]
    fn test_length_bounds() {
        assert!(AzureDevOpsOrganizationName::try_new("a").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("a".repeat(50)).is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("").is_err());
        assert!(AzureDevOpsOrganizationName::try_new("a".repeat(51)).is_err());
    }

    #[test]
    fn test_first_character_rules() {
        assert!(AzureDevOpsOrganizationName::try_new("a").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("A").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("1").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("-abc").is_err());
        assert!(AzureDevOpsOrganizationName::try_new("_abc").is_err());
        assert!(AzureDevOpsOrganizationName::try_new("αbc").is_err()); // Non-English letter
    }

    #[test]
    fn test_last_character_rules() {
        assert!(AzureDevOpsOrganizationName::try_new("abc").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("abc1").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("abc-").is_err());
        assert!(AzureDevOpsOrganizationName::try_new("abc_").is_err());
    }

    #[test]
    fn test_middle_character_rules() {
        assert!(AzureDevOpsOrganizationName::try_new("a-b").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("a1b").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("aBc").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("a_b").is_err());
        assert!(AzureDevOpsOrganizationName::try_new("a.b").is_err());
        assert!(AzureDevOpsOrganizationName::try_new("a b").is_err());
    }

    #[test]
    fn test_english_letters_only() {
        assert!(AzureDevOpsOrganizationName::try_new("ValidOrg").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("αβγ").is_err()); // Greek letters
        assert!(AzureDevOpsOrganizationName::try_new("こんにちは").is_err()); // Japanese
        assert!(AzureDevOpsOrganizationName::try_new("München").is_err()); // German umlaut
    }

    #[test]
    fn test_valid_examples() {
        assert!(AzureDevOpsOrganizationName::try_new("MyOrg").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("org-123").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("Test-Organization-2024").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("a").is_ok());
        assert!(AzureDevOpsOrganizationName::try_new("1").is_ok());
    }

    #[test]
    fn roundtrip_display_and_fromstr() {
        let name = "MyOrg-123";
        let org = AzureDevOpsOrganizationName::from_str(name).unwrap();
        assert_eq!(org.to_string(), name);
        assert_eq!(org.as_ref(), name);
    }

    #[test]
    fn arbitrary_generates_valid_names() {
        for _ in 0..100 {
            let raw: Vec<u8> = (0..128).map(|_| rand::random::<u8>()).collect();
            let mut u = Unstructured::new(&raw);
            if let Ok(org) = AzureDevOpsOrganizationName::arbitrary(&mut u) {
                println!("Generated: {}", org);
                let validation = org.validate();
                assert!(
                    validation.is_ok(),
                    "Arbitrary produced invalid: {:?} - {:?}",
                    &org.inner,
                    validation
                );
            }
        }
    }
}
