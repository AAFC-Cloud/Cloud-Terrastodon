use crate::slug::Slug;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use std::ops::Deref;
use std::str::FromStr;
use validator::Validate;
use validator::ValidationError;

const STORAGE_ACCOUNT_NAMING_RULES_URL: &str = "https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftstorage";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PartialOrd, Ord)]
pub struct StorageAccountBlobContainerName {
    #[validate(length(min = 3, max = 63), custom(function = "validate_name"))]
    inner: CompactString,
}
impl Slug for StorageAccountBlobContainerName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let rtn = Self { inner: name.into() };
        rtn.validate()?;
        Ok(rtn)
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        self.validate()?;
        Ok(())
    }
}
fn validate_name(value: &CompactString) -> Result<(), ValidationError> {
    // Check length requirements (3-63 characters)
    if value.len() < 3 || value.len() > 63 {
        return Err(
            ValidationError::new(STORAGE_ACCOUNT_NAMING_RULES_URL).with_message(
                format!("Value {value:?} must be between 3 and 63 characters long").into(),
            ),
        );
    }

    let chars: Vec<char> = value.chars().collect();

    // Check that it starts with lowercase letter or number
    if let Some(first_char) = chars.first()
        && !first_char.is_ascii_lowercase()
        && !first_char.is_ascii_digit()
    {
        return Err(
            ValidationError::new(STORAGE_ACCOUNT_NAMING_RULES_URL).with_message(
                format!("Value {value:?} must start with a lowercase letter or number").into(),
            ),
        );
    }

    for (i, &char) in chars.iter().enumerate() {
        // Check for valid characters (lowercase letters, numbers, hyphens)
        if !char.is_ascii_lowercase() && !char.is_ascii_digit() && char != '-' {
            return Err(
                ValidationError::new(STORAGE_ACCOUNT_NAMING_RULES_URL).with_message(
                    format!(
                        "Char {char} at position {i} in {value:?} must be lowercase letter, number, or hyphen"
                    )
                    .into(),
                ),
            );
        }

        // Check for consecutive hyphens
        if char == '-' && i > 0 && chars[i - 1] == '-' {
            return Err(
                ValidationError::new(STORAGE_ACCOUNT_NAMING_RULES_URL).with_message(
                    format!(
                        "Value {value:?} cannot contain consecutive hyphens at position {}",
                        i - 1
                    )
                    .into(),
                ),
            );
        }
    }

    Ok(())
}

impl FromStr for StorageAccountBlobContainerName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        StorageAccountBlobContainerName::try_new(s)
    }
}
impl TryFrom<&str> for StorageAccountBlobContainerName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        StorageAccountBlobContainerName::try_new(value)
    }
}
impl std::fmt::Display for StorageAccountBlobContainerName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl serde::Serialize for StorageAccountBlobContainerName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for StorageAccountBlobContainerName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}
impl Deref for StorageAccountBlobContainerName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for StorageAccountBlobContainerName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<StorageAccountBlobContainerName> for CompactString {
    fn from(value: StorageAccountBlobContainerName) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for StorageAccountBlobContainerName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Get length in 3-63
        let len = u.int_in_range(3..=63)?;

        // Characters for first position (must be letter or number)
        let start_choices = ('a'..='z').chain('0'..='9').collect::<Vec<_>>();

        // Characters for remaining positions (letters, numbers, hyphens)
        let all_choices = ('a'..='z')
            .chain('0'..='9')
            .chain(['-'])
            .collect::<Vec<_>>();

        let mut name = String::with_capacity(len);

        // First character must be letter or number
        let first_char = *u.choose(&start_choices)?;
        name.push(first_char);

        // Generate remaining characters
        for _ in 1..len {
            let char = *u.choose(&all_choices)?;

            // Avoid consecutive hyphens
            if char == '-' && name.ends_with('-') {
                // If we would create consecutive hyphens, use a letter instead
                let safe_char = *u.choose(&('a'..='z').collect::<Vec<_>>())?;
                name.push(safe_char);
            } else {
                name.push(char);
            }
        }

        StorageAccountBlobContainerName::try_new(CompactString::from(name))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::StorageAccountBlobContainerName;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::Rng;
    use validator::Validate;

    #[test]
    pub fn validation() -> eyre::Result<()> {
        // Valid cases
        assert!(StorageAccountBlobContainerName::try_new("abc").is_ok()); // minimum length
        assert!(StorageAccountBlobContainerName::try_new("bruh").is_ok()); // lowercase letters
        assert!(StorageAccountBlobContainerName::try_new("123").is_ok()); // numbers only
        assert!(StorageAccountBlobContainerName::try_new("a1b2c3").is_ok()); // letters and numbers
        assert!(StorageAccountBlobContainerName::try_new("a-b-c").is_ok()); // valid hyphens
        assert!(StorageAccountBlobContainerName::try_new("test-container-1").is_ok()); // realistic name
        assert!(StorageAccountBlobContainerName::try_new("1test").is_ok()); // starts with number
        assert!(StorageAccountBlobContainerName::try_new("a".repeat(63)).is_ok()); // maximum length

        // Invalid: too short (less than 3 characters)
        assert!(StorageAccountBlobContainerName::try_new("").is_err());
        assert!(StorageAccountBlobContainerName::try_new("a").is_err());
        assert!(StorageAccountBlobContainerName::try_new("ab").is_err());

        // Invalid: too long (more than 63 characters)
        assert!(StorageAccountBlobContainerName::try_new("a".repeat(64)).is_err());

        // Invalid: starts with hyphen
        assert!(StorageAccountBlobContainerName::try_new("-abc").is_err());
        assert!(StorageAccountBlobContainerName::try_new("-123").is_err());

        // Invalid: consecutive hyphens
        assert!(StorageAccountBlobContainerName::try_new("a--b").is_err());
        assert!(StorageAccountBlobContainerName::try_new("test--container").is_err());
        assert!(StorageAccountBlobContainerName::try_new("a---b").is_err());

        // Invalid: uppercase letters
        assert!(StorageAccountBlobContainerName::try_new("JEOFF").is_err());
        assert!(StorageAccountBlobContainerName::try_new("caPital").is_err());
        assert!(StorageAccountBlobContainerName::try_new("Test").is_err());
        assert!(StorageAccountBlobContainerName::try_new("testA").is_err());

        // Invalid: special characters
        assert!(StorageAccountBlobContainerName::try_new("hi+hi").is_err());
        assert!(StorageAccountBlobContainerName::try_new("test_container").is_err());
        assert!(StorageAccountBlobContainerName::try_new("test.container").is_err());
        assert!(StorageAccountBlobContainerName::try_new("test@container").is_err());
        assert!(StorageAccountBlobContainerName::try_new("test container").is_err()); // space

        Ok(())
    }

    #[test]
    #[ignore]
    pub fn preview_failure() -> eyre::Result<()> {
        StorageAccountBlobContainerName::try_new("abc123B321")?; // Should fail due to uppercase 'B'
        Ok(())
    }

    #[test]
    pub fn fuzz() -> eyre::Result<()> {
        for _ in 0..100 {
            let mut raw = [0u8; 64];
            rand::thread_rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = StorageAccountBlobContainerName::arbitrary(&mut un)?;
            assert!(name.validate().is_ok());
            println!("{name}");
        }
        Ok(())
    }
}
