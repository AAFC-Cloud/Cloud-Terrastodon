use crate::slug::Slug;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use std::ops::Deref;
use std::str::FromStr;
use validator::Validate;
use validator::ValidationError;

/// Azure Key Vault Secret name constraints
/// * Length: 1-127 characters
/// * Allowed characters: alphanumerics (A-Z, a-z, 0-9) and hyphen '-'
/// * Case-sensitive (we keep the exact casing) but validation only checks allowed set
/// (Docs excerpt provided by user: "vaults / secrets Vault 1-127 Alphanumerics and hyphens")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PartialOrd, Ord)]
pub struct KeyVaultSecretName {
    #[validate(length(min = 1, max = 127), custom(function = "validate_secret_name"))]
    inner: CompactString,
}

impl Slug for KeyVaultSecretName {
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

fn validate_secret_name(value: &CompactString) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::new("keyvaultsecretname_empty"));
    }
    if value.len() > 127 {
        return Err(ValidationError::new("keyvaultsecretname_length")
            .with_message(format!("Secret name too long: {} > 127", value.len()).into()));
    }
    for (i, ch) in value.chars().enumerate() {
        if !(ch.is_ascii_alphanumeric() || ch == '-') {
            return Err(
                ValidationError::new("keyvaultsecretname_char").with_message(
                    format!("Invalid character '{ch}' at position {i} in {value:?}").into(),
                ),
            );
        }
    }
    Ok(())
}

impl std::fmt::Display for KeyVaultSecretName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl FromStr for KeyVaultSecretName {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        KeyVaultSecretName::try_new(s)
    }
}
impl TryFrom<&str> for KeyVaultSecretName {
    type Error = eyre::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl Deref for KeyVaultSecretName {
    type Target = CompactString;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl serde::Serialize for KeyVaultSecretName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}
impl<'de> serde::Deserialize<'de> for KeyVaultSecretName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}
impl TryFrom<CompactString> for KeyVaultSecretName {
    type Error = eyre::Error;
    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<KeyVaultSecretName> for CompactString {
    fn from(v: KeyVaultSecretName) -> Self {
        v.inner
    }
}

impl<'a> Arbitrary<'a> for KeyVaultSecretName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(1..=32)?; // keep fuzz small while within valid range
        let mut s = String::with_capacity(len as usize);
        while s.len() < len as usize {
            let b: u8 = u.arbitrary()?;
            let ch = match b % 63 {
                // 0..=62
                n if n < 10 => (b'0' + n) as char,
                n if n < 36 => (b'A' + (n - 10)) as char,
                n if n < 62 => (b'a' + (n - 36)) as char,
                _ => '-',
            };
            if ch == '-' && s.is_empty() {
                continue;
            } // allow leading hyphen per docs? docs say just allowed set; keep simple
            s.push(ch);
        }
        KeyVaultSecretName::try_new(CompactString::from(s))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;

    #[test]
    fn validation() -> eyre::Result<()> {
        assert!(KeyVaultSecretName::try_new("a").is_ok());
        assert!(KeyVaultSecretName::try_new("A-1-z").is_ok());
        assert!(KeyVaultSecretName::try_new("a".repeat(127)).is_ok());
        assert!(KeyVaultSecretName::try_new("").is_err());
        assert!(KeyVaultSecretName::try_new("a".repeat(128)).is_err());
        assert!(KeyVaultSecretName::try_new("bad_").is_err());
        assert!(KeyVaultSecretName::try_new("bad.").is_err());
        Ok(())
    }

    #[test]
    fn fuzz() -> eyre::Result<()> {
        for _ in 0..100 {
            let mut raw = [0u8; 64];
            rand::thread_rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = KeyVaultSecretName::arbitrary(&mut un)?;
            name.validate_slug()?;
        }
        Ok(())
    }
}
