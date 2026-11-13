use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fmt;
use std::str::FromStr;

/// Case-insensitive Key Vault key name: 1–127 ASCII letters, digits, or hyphens.
///
/// [Key Vault object identifiers](https://learn.microsoft.com/en-us/azure/key-vault/general/about-keys-secrets-certificates#object-identifiers).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(proxy = String)]
pub struct AzureKeyVaultKeyName {
    // All construction validates the name and normalizes its case-insensitive identity.
    value: String,
}

impl FromStr for AzureKeyVaultKeyName {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        ensure!(
            (1..=127).contains(&value.len())
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-'),
            "Key Vault key names require 1–127 ASCII letters, digits, or hyphens"
        );
        Ok(Self {
            value: value.to_ascii_lowercase(),
        })
    }
}

impl fmt::Display for AzureKeyVaultKeyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

impl<'a> Arbitrary<'a> for AzureKeyVaultKeyName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            value: format!("synthetic-key-{}", u.arbitrary::<u64>()?),
        })
    }
}

crate::impl_facet_string_proxy!(AzureKeyVaultKeyName, value => value.to_string());
cloud_terrastodon_registry::register_thing!(AzureKeyVaultKeyName);
cloud_terrastodon_registry::register_arbitrary!(AzureKeyVaultKeyName);
