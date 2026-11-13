use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::WrapErr;
use eyre::ensure;
use facet::Facet;
use std::str::FromStr;
use url::Url;

/// A service-assigned storage endpoint, including sovereign-cloud and custom hosts.
///
/// [Storage Resource Provider 2025-06-01 endpoints](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#endpoints).
/// The service owns the hostname and route; neither identifies an ARM resource.
/// This library requires an absolute HTTP(S) URL, without inferring an account
/// identity from a DNS suffix or restricting service-provided paths.
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
#[facet(proxy = String)]
pub struct AzureStorageAccountServiceEndpoint {
    // Private so callers cannot replace a valid endpoint with a non-HTTP URL.
    url: Url,
}

impl AzureStorageAccountServiceEndpoint {
    pub fn as_url(&self) -> &Url {
        &self.url
    }
}

impl FromStr for AzureStorageAccountServiceEndpoint {
    type Err = eyre::Report;

    fn from_str(value: &str) -> eyre::Result<Self> {
        let url = Url::parse(value).wrap_err("invalid storage service endpoint URL")?;
        ensure!(
            matches!(url.scheme(), "http" | "https") && url.has_host(),
            "storage service endpoint must be an absolute HTTP(S) URL"
        );
        Ok(Self { url })
    }
}

impl TryFrom<String> for AzureStorageAccountServiceEndpoint {
    type Error = eyre::Report;

    fn try_from(value: String) -> eyre::Result<Self> {
        value.parse()
    }
}

impl From<&AzureStorageAccountServiceEndpoint> for String {
    fn from(value: &AzureStorageAccountServiceEndpoint) -> Self {
        value.url.to_string()
    }
}

impl<'a> Arbitrary<'a> for AzureStorageAccountServiceEndpoint {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        format!(
            "https://synthetic{}.blob.core.windows.net/",
            u.arbitrary::<u32>()?
        )
        .parse()
        .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountServiceEndpoint);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountServiceEndpoint);

#[cfg(test)]
mod tests {
    use super::AzureStorageAccountServiceEndpoint;

    #[test]
    fn preserves_service_routes_without_assuming_public_cloud_hostnames() -> eyre::Result<()> {
        for value in [
            "https://synthetic.blob.core.usgovcloudapi.net/",
            "https://synthetic.z42.web.core.windows.net/",
            "http://storage.example.test/custom/route/",
        ] {
            assert_eq!(
                value
                    .parse::<AzureStorageAccountServiceEndpoint>()?
                    .as_url()
                    .as_str(),
                value
            );
        }
        for value in [
            "/relative/path",
            "file:///storage",
            "mailto:storage@example.test",
        ] {
            assert!(value.parse::<AzureStorageAccountServiceEndpoint>().is_err());
        }
        Ok(())
    }
}
