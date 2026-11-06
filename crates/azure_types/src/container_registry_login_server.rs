use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::str::FromStr;
use url::Url;

/// Azure public-cloud registry DNS identity, including domain-name-label hashes.
/// Read the service's loginServer property instead of deriving it from a registry name.
/// [Registry properties](https://learn.microsoft.com/en-us/rest/api/container-registry/registries/get?view=rest-container-registry-2025-11-01#registry).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(proxy = String)]
pub struct ContainerRegistryLoginServer(String);

impl ContainerRegistryLoginServer {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn endpoint(&self) -> Result<Url> {
        Ok(Url::parse(&format!("https://{}/", self.0))?)
    }
}
impl FromStr for ContainerRegistryLoginServer {
    type Err = eyre::Report;
    fn from_str(value: &str) -> Result<Self> {
        let url = Url::parse(&format!("https://{value}/"))?;
        let host = url
            .host_str()
            .ok_or_else(|| eyre::eyre!("registry login server must have a hostname"))?;
        ensure!(
            host.strip_suffix(".azurecr.io")
                .is_some_and(|prefix| !prefix.is_empty())
                && url.username().is_empty()
                && url.password().is_none()
                && url.port().is_none()
                && url.path() == "/"
                && url.query().is_none()
                && url.fragment().is_none()
                && value.eq_ignore_ascii_case(host),
            "registry login server must be an Azure public-cloud registry hostname"
        );
        Ok(Self(host.to_owned()))
    }
}
impl TryFrom<String> for ContainerRegistryLoginServer {
    type Error = eyre::Report;
    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}
impl From<&ContainerRegistryLoginServer> for String {
    fn from(value: &ContainerRegistryLoginServer) -> Self {
        value.0.clone()
    }
}
impl<'a> Arbitrary<'a> for ContainerRegistryLoginServer {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(format!("fixture{}.azurecr.io", u16::arbitrary(u)?)))
    }
}
cloud_terrastodon_registry::register_thing!(ContainerRegistryLoginServer);
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistryLoginServer);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_service_assigned_hostnames_without_admitting_url_components() {
        let server: ContainerRegistryLoginServer = "Example-ab12.azurecr.io".parse().unwrap();
        assert_eq!(server.as_str(), "example-ab12.azurecr.io");
        for value in [
            "example.azurecr.io.evil.test",
            "user@example.azurecr.io",
            "example.azurecr.io/path",
            "example.azurecr.io:443",
            "example.azurecr.io?x=1",
            "azurecr.io",
        ] {
            assert!(
                value.parse::<ContainerRegistryLoginServer>().is_err(),
                "{value}"
            );
        }
    }
}
