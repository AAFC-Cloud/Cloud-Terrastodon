use crate::BmcHelixServerId;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use facet::Facet;
use std::fmt;
use std::str::FromStr;
use url::Url;

/// An HTTPS origin hosting the [Helix AR System REST API](https://docs.helixops.ai/bin/Service-Management/Innovation-Suite/BMC-Helix-Innovation-Suite/is254/Developing-an-API-program/Integrating-AR-System-forms-with-a-third-party-application-by-using-the-REST-API/).
///
/// Cloud Terrastodon supports a root URL with an optional port. Paths and
/// embedded credentials are excluded so the URL can identify the server's sessions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[facet(proxy = String)]
pub struct BmcHelixServerUrl(Url);

impl BmcHelixServerUrl {
    /// Returns the validated server origin with its trailing slash.
    pub fn as_url(&self) -> &Url {
        &self.0
    }

    /// Identifies this server independently of the name used to save it.
    pub fn id(&self) -> BmcHelixServerId {
        BmcHelixServerId::for_url(self)
    }
}

impl FromStr for BmcHelixServerUrl {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        let url = Url::parse(value).wrap_err("parsing the Helix server URL")?;
        Self::try_from(url)
    }
}

impl TryFrom<Url> for BmcHelixServerUrl {
    type Error = eyre::Report;

    fn try_from(url: Url) -> Result<Self> {
        ensure!(
            url.scheme() == "https"
                && url.host_str().is_some()
                && url.path() == "/"
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none(),
            "Helix server URL must be an HTTPS origin, such as https://helix.example.com"
        );
        Ok(Self(url))
    }
}

impl TryFrom<String> for BmcHelixServerUrl {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl From<&BmcHelixServerUrl> for String {
    fn from(value: &BmcHelixServerUrl) -> Self {
        value.0.as_str().to_owned()
    }
}

impl fmt::Display for BmcHelixServerUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0.as_str())
    }
}

impl<'a> Arbitrary<'a> for BmcHelixServerUrl {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        format!("https://helix-{}.example.invalid", u32::arbitrary(input)?)
            .parse()
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(BmcHelixServerUrl);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixServerUrl);

#[cfg(test)]
mod tests {
    use super::BmcHelixServerUrl;

    #[test]
    fn server_urls_canonicalize_origins() -> eyre::Result<()> {
        let plain: BmcHelixServerUrl = "https://helix.example.com".parse()?;
        let with_default_port: BmcHelixServerUrl = "https://HELIX.example.com:443/".parse()?;
        let with_custom_port: BmcHelixServerUrl = "https://helix.example.com:8443/".parse()?;
        assert_eq!(plain, with_default_port);
        assert_ne!(plain, with_custom_port);
        assert_eq!(plain.id(), with_default_port.id());
        assert_ne!(plain.id(), with_custom_port.id());
        assert_eq!(
            plain.id().to_string().parse::<crate::BmcHelixServerId>()?,
            plain.id()
        );
        Ok(())
    }

    #[test]
    fn server_urls_exclude_credentials_and_routes() {
        for value in [
            "http://helix.example.com",
            "https://user:secret@helix.example.com",
            "https://helix.example.com/api/",
            "https://helix.example.com/?token=secret",
            "https://helix.example.com/#fragment",
        ] {
            assert!(value.parse::<BmcHelixServerUrl>().is_err(), "{value}");
        }
    }
}
