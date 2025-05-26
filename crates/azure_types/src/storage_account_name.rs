use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use serde::de::Error;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;
use validator::Validate;
use validator::ValidationError;

use crate::slug::Slug;

const STORAGE_ACCOUNT_NAMING_RULES_URL: &str = "https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftstorage";
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PartialOrd, Ord)]
pub struct StorageAccountName {
    #[validate(
        length(min = 3, max = 24),
        custom(function = "validate_lowercase_alphanumeric")
    )]
    pub inner: CompactString,
}
impl Slug for StorageAccountName {
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
fn validate_lowercase_alphanumeric(value: &CompactString) -> Result<(), ValidationError> {
    for (i, char) in value.chars().enumerate() {
        if !char.is_ascii_alphanumeric() {
            return Err(
                ValidationError::new(STORAGE_ACCOUNT_NAMING_RULES_URL).with_message(
                    format!(
                        "Char {char} as position {i} in {value:?} must be lowercase alphanumeric"
                    )
                    .into(),
                ),
            );
        }
        if char.is_uppercase() {
            return Err(
                ValidationError::new(STORAGE_ACCOUNT_NAMING_RULES_URL).with_message(
                    format!("Char {char} as position {i} in {value:?} must be lowercase").into(),
                ),
            );
        }
    }
    Ok(())
}

impl FromStr for StorageAccountName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        StorageAccountName::try_new(s)
    }
}
impl std::fmt::Display for StorageAccountName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl serde::Serialize for StorageAccountName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for StorageAccountName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| D::Error::custom(format!("{e:?}")))
    }
}
impl Deref for StorageAccountName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for StorageAccountName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl TryFrom<CompactString> for StorageAccountName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<StorageAccountName> for CompactString {
    fn from(value: StorageAccountName) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for StorageAccountName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Get length in 3-24
        let len = u.int_in_range(3..=24)?;
        // Use only [a-z]
        let choices = ('a'..='z').chain('1'..='9').collect::<Vec<_>>();
        let name: String = (0..len)
            .map(|_| {
                // Safe since 'a'..'z' is always valid
                let c = u.choose(&choices)?;
                Ok(*c)
            })
            .collect::<Result<String, _>>()?;
        StorageAccountName::try_new(CompactString::from(name))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::StorageAccountName;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::Rng;
    use validator::Validate;

    #[test]
    pub fn validation() -> eyre::Result<()> {
        assert!(StorageAccountName::try_new("bruh").is_ok());
        assert!(StorageAccountName::try_new("-").is_err());
        assert!(StorageAccountName::try_new("a-b-c").is_err());
        assert!(StorageAccountName::try_new("hi+hi").is_err());
        assert!(StorageAccountName::try_new("").is_err());
        assert!(StorageAccountName::try_new("a").is_err());
        assert!(StorageAccountName::try_new("aa").is_err());
        assert!(StorageAccountName::try_new("JEOFF").is_err());
        assert!(StorageAccountName::try_new("caPital").is_err());
        assert!(StorageAccountName::try_new("aaaa").is_ok());
        assert!(StorageAccountName::try_new("a".repeat(23)).is_ok());
        assert!(StorageAccountName::try_new("a".repeat(24)).is_ok());
        assert!(StorageAccountName::try_new("a".repeat(25)).is_err());
        Ok(())
    }

    #[test]
    #[ignore]
    pub fn preview_failure() -> eyre::Result<()> {
        StorageAccountName::try_new("abc123B321")?;
        Ok(())
    }

    #[test]
    pub fn fuzz() -> eyre::Result<()> {
        for _ in 0..100 {
            let mut raw = [0u8; 64];
            rand::thread_rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = StorageAccountName::arbitrary(&mut un)?;
            assert!(name.validate().is_ok());
            println!("{name}");
        }
        Ok(())
    }
}
