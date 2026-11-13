use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fmt;
use std::str::FromStr;
use url::Url;

/// Base HTTPS endpoint for a Key Vault or Managed HSM, independent of cloud suffix.
///
/// [Key Vault endpoint forms](https://learn.microsoft.com/en-us/azure/key-vault/general/about-keys-secrets-certificates#dns-suffixes-for-object-identifiers).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(proxy = String)]
pub struct AzureKeyVaultEndpoint {
    // A base endpoint has no object route, query, fragment, or embedded credentials.
    url: Url,
}

impl AzureKeyVaultEndpoint {
    pub fn to_url(&self) -> Url {
        self.url.clone()
    }
}

impl FromStr for AzureKeyVaultEndpoint {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        let url = Url::parse(value).wrap_err("parsing the Key Vault endpoint")?;
        ensure!(
            url.scheme() == "https"
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
                && url.path() == "/"
                && url.query().is_none()
                && url.fragment().is_none(),
            "Key Vault endpoint must be an HTTPS service root without credentials, query, or fragment"
        );
        Ok(Self { url })
    }
}

impl fmt::Display for AzureKeyVaultEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.url.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureKeyVaultEndpoint {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        format!(
            "https://synthetic{}.vault.azure.net/",
            u.arbitrary::<u32>()?
        )
        .parse()
        .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

crate::impl_facet_string_proxy!(AzureKeyVaultEndpoint, value => value.to_string());
cloud_terrastodon_registry::register_thing!(AzureKeyVaultEndpoint);
cloud_terrastodon_registry::register_arbitrary!(AzureKeyVaultEndpoint);
