use crate::slug::Slug;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use eyre::Context;
use eyre::bail;
use facet::Facet;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Facet)]
#[facet(json::proxy = String)]
pub struct AzureStorageAccountName {
    inner: CompactString,
}
crate::impl_facet_string_proxy!(AzureStorageAccountName, value => value.to_string());
impl Slug for AzureStorageAccountName {
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

impl FromStr for AzureStorageAccountName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AzureStorageAccountName::try_new(s)
    }
}
impl TryFrom<&str> for AzureStorageAccountName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        AzureStorageAccountName::try_new(value)
    }
}
impl std::fmt::Display for AzureStorageAccountName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl Deref for AzureStorageAccountName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for AzureStorageAccountName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<AzureStorageAccountName> for CompactString {
    fn from(value: AzureStorageAccountName) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for AzureStorageAccountName {
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
        AzureStorageAccountName::try_new(CompactString::from(name))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountName);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountName);

#[cfg(test)]
mod test {
    use crate::AzureStorageAccountName;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::RngExt;

    #[test]
    pub fn validation() -> eyre::Result<()> {
        assert!(AzureStorageAccountName::try_new("bruh").is_ok());
        assert!(AzureStorageAccountName::try_new("-").is_err());
        assert!(AzureStorageAccountName::try_new("a-b-c").is_err());
        assert!(AzureStorageAccountName::try_new("hi+hi").is_err());
        assert!(AzureStorageAccountName::try_new("").is_err());
        assert!(AzureStorageAccountName::try_new("a").is_err());
        assert!(AzureStorageAccountName::try_new("aa").is_err());
        assert!(AzureStorageAccountName::try_new("JEOFF").is_err());
        assert!(AzureStorageAccountName::try_new("caPital").is_err());
        assert!(AzureStorageAccountName::try_new("aaaa").is_ok());
        assert!(AzureStorageAccountName::try_new("a".repeat(23)).is_ok());
        assert!(AzureStorageAccountName::try_new("a".repeat(24)).is_ok());
        assert!(AzureStorageAccountName::try_new("a".repeat(25)).is_err());
        Ok(())
    }

    #[test]
    #[ignore]
    pub fn preview_failure() -> eyre::Result<()> {
        AzureStorageAccountName::try_new("abc123B321")?;
        Ok(())
    }

    #[test]
    pub fn fuzz() -> eyre::Result<()> {
        for _ in 0..100 {
            let mut raw = [0u8; 64];
            rand::rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = AzureStorageAccountName::arbitrary(&mut un)?;
            assert!(name.validate_slug().is_ok());
        }
        Ok(())
    }

    #[test]
    pub fn json_round_trips_through_facet() -> eyre::Result<()> {
        let name = facet_json::from_str::<AzureStorageAccountName>("\"bruh\"")?;
        assert_eq!(name, AzureStorageAccountName::try_new("bruh")?);
        assert_eq!(facet_json::to_string(&name)?, "\"bruh\"");
        Ok(())
    }
}
