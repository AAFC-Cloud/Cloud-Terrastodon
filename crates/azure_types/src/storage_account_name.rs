use crate::slug::Slug;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use eyre::Context;
use eyre::bail;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, facet::Facet)]
#[facet(json::proxy = String)]
pub struct StorageAccountName {
    inner: CompactString,
}
crate::impl_facet_string_proxy!(StorageAccountName, value => value.to_string());
impl Slug for StorageAccountName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        validate_storage_account_name(&inner)?;
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        validate_storage_account_name(&self.inner)?;
        Ok(())
    }
}
fn validate_storage_account_name(value: &CompactString) -> eyre::Result<()> {
    validate_storage_account_name_inner(value)
        .wrap_err_with(|| format!("Invalid storage account name: {}", value))
        .wrap_err("https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftstorage")
}

fn validate_storage_account_name_inner(value: &CompactString) -> eyre::Result<()> {
    let char_count = value.chars().count();
    if !(3..=24).contains(&char_count) {
        bail!("Storage account name must be between 3 and 24 characters");
    }
    for (i, char) in value.chars().enumerate() {
        if !char.is_ascii_alphanumeric() {
            bail!(
                "Char {} at position {} in {:?} must be lowercase alphanumeric",
                char,
                i,
                value
            );
        }
        if char.is_uppercase() {
            bail!(
                "Char {} at position {} in {:?} must be lowercase",
                char,
                i,
                value
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
impl TryFrom<&str> for StorageAccountName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        StorageAccountName::try_new(value)
    }
}
impl std::fmt::Display for StorageAccountName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl Deref for StorageAccountName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
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

cloud_terrastodon_registry::register_thing!(StorageAccountName);

#[cfg(test)]
mod test {
    use crate::StorageAccountName;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::RngExt;

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
            rand::rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = StorageAccountName::arbitrary(&mut un)?;
            assert!(name.validate_slug().is_ok());
        }
        Ok(())
    }

    #[test]
    pub fn json_round_trips_through_facet() -> eyre::Result<()> {
        let name = facet_json::from_str::<StorageAccountName>("\"bruh\"")?;
        assert_eq!(name, StorageAccountName::try_new("bruh")?);
        assert_eq!(facet_json::to_string(&name)?, "\"bruh\"");
        Ok(())
    }
}

