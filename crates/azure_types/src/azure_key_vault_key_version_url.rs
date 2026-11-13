use crate::AzureKeyVaultEndpoint;
use crate::AzureKeyVaultKeyName;
use crate::PercentDecodeExt;
use crate::PercentEncodeExt;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fmt;
use std::str::FromStr;
use url::Url;

/// Versioned key identity returned by storage encryption metadata.
///
/// [Key Vault key route](https://learn.microsoft.com/en-us/rest/api/keyvault/keys/get-key/get-key?view=rest-keyvault-keys-2025-07-01).
/// The vault endpoint retains sovereign-cloud and Managed HSM hosts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(proxy = String)]
pub struct AzureKeyVaultKeyVersionUrl {
    pub vault: AzureKeyVaultEndpoint,
    pub key_name: AzureKeyVaultKeyName,
    // Keep the opaque version separate from URL encoding. Mutation could introduce
    // a slash or remove the version, turning this into a different resource route.
    version: String,
}

impl AzureKeyVaultKeyVersionUrl {
    /// The decoded version identifier, preserving its original spelling.
    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn to_url(&self) -> Result<Url> {
        Url::parse(&self.to_string()).wrap_err("assembling the versioned Key Vault key URL")
    }
}

impl FromStr for AzureKeyVaultKeyVersionUrl {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        let mut url = Url::parse(value).wrap_err("parsing the versioned Key Vault key URL")?;
        let parts = url
            .path()
            .strip_prefix('/')
            .ok_or_else(|| eyre::eyre!("Key Vault key URL requires an absolute object path"))?
            .split('/')
            .collect::<Vec<_>>();
        let [kind, name, version] = parts.as_slice() else {
            return Err(eyre::eyre!(
                "Key Vault key URL must contain /keys/name/version"
            ));
        };
        let kind = kind
            .percent_decode()
            .wrap_err("decoding Key Vault object type")?;
        let name = name
            .percent_decode()
            .wrap_err("decoding Key Vault key name")?;
        let version = version
            .percent_decode()
            .wrap_err("decoding Key Vault key version")?;
        ensure!(
            kind.eq_ignore_ascii_case("keys") && !version.is_empty() && !version.contains('/'),
            "Key Vault key URL must contain /keys/name/version"
        );
        let key_name = name.parse()?;
        let version = version.into_owned();
        url.set_path("/");
        let vault = url.as_str().parse()?;
        Ok(Self {
            vault,
            key_name,
            version,
        })
    }
}

impl fmt::Display for AzureKeyVaultKeyVersionUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}keys/{}/{}",
            self.vault,
            self.key_name,
            self.version.percent_encode()
        )
    }
}

impl<'a> Arbitrary<'a> for AzureKeyVaultKeyVersionUrl {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            vault: u.arbitrary()?,
            key_name: u.arbitrary()?,
            version: format!("{:032x}", u.arbitrary::<u128>()?),
        })
    }
}

crate::impl_facet_string_proxy!(AzureKeyVaultKeyVersionUrl, value => value.to_string());
cloud_terrastodon_registry::register_thing!(AzureKeyVaultKeyVersionUrl);
cloud_terrastodon_registry::register_arbitrary!(AzureKeyVaultKeyVersionUrl);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versioned_identity_preserves_authority_and_rejects_other_routes() -> Result<()> {
        let key: AzureKeyVaultKeyVersionUrl =
            "https://example.managedhsm.azure.cn/keys/MixedCase/abcdef0123456789abcdef0123456789"
                .parse()?;
        assert_eq!(
            key.vault.to_string(),
            "https://example.managedhsm.azure.cn/"
        );
        assert_eq!(key.key_name.to_string(), "mixedcase");
        assert_eq!(
            key.to_url()?.host_str(),
            Some("example.managedhsm.azure.cn")
        );
        let encoded: AzureKeyVaultKeyVersionUrl =
            "https://example.managedhsm.azure.cn/keys/%4DixedCase/ab%43d%2Bversion".parse()?;
        assert_eq!(encoded.key_name, key.key_name);
        assert_eq!(encoded.version(), "abCd+version");
        assert_eq!(
            encoded.to_string().parse::<AzureKeyVaultKeyVersionUrl>()?,
            encoded
        );
        let literal_escape: AzureKeyVaultKeyVersionUrl =
            "https://example.vault.azure.net/keys/name/%252E".parse()?;
        assert_eq!(literal_escape.version(), "%2E");
        assert_eq!(
            literal_escape
                .to_string()
                .parse::<AzureKeyVaultKeyVersionUrl>()?,
            literal_escape
        );
        for invalid in [
            "https://example.vault.azure.net/secrets/name/version",
            "https://example.vault.azure.net/keys/name",
            "https://example.vault.azure.net/keys/name/version?query=1",
            "https://user@example.vault.azure.net/keys/name/version",
            "https://example.vault.azure.net//keys/name/version",
            "https://example.vault.azure.net/keys/name%2Fchild/version",
            "https://example.vault.azure.net/keys/name/version%2Fchild",
            "https://example.vault.azure.net/keys/name/%2E",
            "https://example.vault.azure.net/keys/name/%2E%2E",
            "https://example.vault.azure.net/keys/name/%FF",
            "https://example.vault.azure.net/keys/name/version%",
        ] {
            assert!(
                invalid.parse::<AzureKeyVaultKeyVersionUrl>().is_err(),
                "{invalid}"
            );
        }
        Ok(())
    }
}
