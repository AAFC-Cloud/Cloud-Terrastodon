use crate::slug::Slug;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use std::hash::Hash;
use std::ops::Deref;
use std::str::FromStr;
use validator::Validate;
use validator::ValidationError;

/// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftkeyvault
/// Constraints (Azure Key Vault name):
///  * Length 3-24 characters
///  * Allowed characters: ASCII letters, digits, and hyphen '-'
///  * Must start with a letter
///  * Must end with a letter or digit
///  * Cannot contain consecutive hyphens
///  * Case-insensitive (comparisons and hashing lower-case)
/// 
/// See also: https://github.com/Azure/azure-rest-api-specs/blob/6c548b0bd279f5e233661b1c81fb5b61b19965cd/specification/keyvault/resource-manager/Microsoft.KeyVault/stable/2024-11-01/keyvault.json
#[derive(Debug, Clone, Eq, Validate, PartialOrd, Ord)]
pub struct KeyVaultName {
    #[validate(
        length(min = 3, max = 24),
        custom(function = "validate_key_vault_name_contents")
    )]
    inner: CompactString,
}
impl PartialEq for KeyVaultName {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq_ignore_ascii_case(&other.inner)
    }
}
impl Hash for KeyVaultName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.to_ascii_lowercase().hash(state);
    }
}
impl Slug for KeyVaultName {
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

impl FromStr for KeyVaultName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        KeyVaultName::try_new(s)
    }
}
impl TryFrom<&str> for KeyVaultName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        KeyVaultName::try_new(value)
    }
}
impl TryFrom<String> for KeyVaultName {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}
impl TryFrom<&String> for KeyVaultName {
    type Error = eyre::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

fn validate_key_vault_name_contents(value: &CompactString) -> Result<(), ValidationError> {
    let s = value.as_str();
    if s.is_empty() {
        return Err(ValidationError::new("keyvaultname_empty"));
    }
    // Start with a letter
    let first = s.chars().next().unwrap();
    if !first.is_ascii_alphabetic() {
        return Err(ValidationError::new("keyvaultname_start")
            .with_message(format!("Key Vault name must start with a letter: {s:?}").into()));
    }
    // Allowed chars and no consecutive hyphens
    let mut prev_hyphen = false;
    for (i, ch) in s.chars().enumerate() {
        if ch == '-' {
            if prev_hyphen {
                return Err(
                    ValidationError::new("keyvaultname_consecutive_hyphens").with_message(
                        format!("Consecutive hyphens at position {}-{i} in {s:?}", i - 1).into(),
                    ),
                );
            }
            prev_hyphen = true;
            continue;
        }
        prev_hyphen = false;
        if !ch.is_ascii_alphanumeric() {
            return Err(ValidationError::new("keyvaultname_char").with_message(
                format!("Invalid character '{ch}' at position {i} in {s:?}").into(),
            ));
        }
    }
    // End with letter or digit (not hyphen already covered) but also must not end with hyphen
    let last = s.chars().last().unwrap();
    if !(last.is_ascii_alphanumeric()) {
        return Err(ValidationError::new("keyvaultname_end").with_message(
            format!("Key Vault name must end with a letter or digit: {s:?}").into(),
        ));
    }
    Ok(())
}

impl std::fmt::Display for KeyVaultName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl serde::Serialize for KeyVaultName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for KeyVaultName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}
impl Deref for KeyVaultName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for KeyVaultName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<KeyVaultName> for CompactString {
    fn from(value: KeyVaultName) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for KeyVaultName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(3..=24)?;
        debug_assert!(len >= 3);
        // First char: letter
        let first = if u.arbitrary::<bool>()? {
            // lower
            (b'a' + (u.int_in_range(0..=25)? as u8)) as char
        } else {
            (b'A' + (u.int_in_range(0..=25)? as u8)) as char
        };
        let mut s = String::with_capacity(len as usize);
        s.push(first);
        let mut prev_hyphen = false;
        while s.len() < len as usize - 1 {
            // leave space for last char decision
            let choice: u8 = u.arbitrary()?; // random byte
            let ch = match choice % 63 {
                // enough spread
                0 => '-',
                n if n < 10 + 1 => (b'0' + (n - 1)) as char, // digits bucket
                n if n < 10 + 1 + 26 => (b'a' + (n - 11)) as char,
                n if n < 10 + 1 + 26 + 26 => (b'A' + (n - 37)) as char,
                _ => 'a',
            };
            if ch == '-' {
                if prev_hyphen {
                    continue;
                }
                // Don't allow hyphen as penultimate if last will also be hyphen (we'll ensure last not hyphen anyway)
                prev_hyphen = true;
                s.push('-');
            } else {
                prev_hyphen = false;
                s.push(ch);
            }
        }
        // Last char: letter or digit (not hyphen)
        let last_bucket: u8 = u.arbitrary::<u8>()? % 62; // 0-61
        let last = match last_bucket {
            n if n < 10 => (b'0' + n) as char,
            n if n < 36 => (b'a' + (n - 10)) as char,
            n => (b'A' + (n - 36)) as char,
        };
        s.push(last);
        debug_assert!(s.len() == len as usize);
        KeyVaultName::try_new(CompactString::from(s)).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::Rng;

    #[test]
    pub fn validation() -> eyre::Result<()> {
        // valid basic
        assert!(KeyVaultName::try_new("abc").is_ok());
        assert!(KeyVaultName::try_new("a-b-c").is_ok());
        assert!(KeyVaultName::try_new("A1-b2-C3").is_ok());
        // length boundaries
        assert!(KeyVaultName::try_new("ab").is_err()); // too short (<3)
        assert!(KeyVaultName::try_new("a".repeat(24).as_str()).is_ok());
        assert!(KeyVaultName::try_new(&("a".repeat(25))).is_err());
        // start constraints
        assert!(KeyVaultName::try_new("1abc").is_err()); // starts with digit
        assert!(KeyVaultName::try_new("-abc").is_err()); // starts with hyphen
        // end constraints
        assert!(KeyVaultName::try_new("abc-").is_err());
        // allowed charset
        assert!(KeyVaultName::try_new("abC123").is_ok());
        assert!(KeyVaultName::try_new("ab_c").is_err()); // underscore not allowed
        assert!(KeyVaultName::try_new("ab.c").is_err()); // period not allowed
        assert!(KeyVaultName::try_new("ab+c").is_err());
        // consecutive hyphens
        assert!(KeyVaultName::try_new("ab--cd").is_err());
        assert!(KeyVaultName::try_new("a--b").is_err());
        // case insensitivity check
        let a = KeyVaultName::try_new("MyVault")?;
        let b = KeyVaultName::try_new("myvault")?;
        assert_eq!(a, b);
        Ok(())
    }

    #[test]
    pub fn fuzz() -> eyre::Result<()> {
        for _ in 0..100 {
            let mut raw = [0u8; 128];
            rand::thread_rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = KeyVaultName::arbitrary(&mut un)?;
            assert!(name.validate_slug().is_ok());
            println!("{name}");
        }
        Ok(())
    }
}
