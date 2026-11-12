use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fmt;
use std::str::FromStr;
use url::Url;

/// Validated service instance defining an account-alias namespace.
///
/// Keeps scheme, authority, and deployment path, including on-premises instances.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Facet)]
#[facet(json::proxy = String)]
pub struct CloudTerrastodonAccountService(Url);
impl CloudTerrastodonAccountService {
    pub fn new(mut url: Url) -> Result<Self> {
        ensure!(
            matches!(url.scheme(), "http" | "https") && url.host_str().is_some(),
            "account service must be an absolute HTTP or HTTPS URL"
        );
        ensure!(
            url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none(),
            "account service URL cannot contain credentials, a query, or a fragment"
        );
        let path = url.path().trim_end_matches('/').to_owned();
        url.set_path(&path);
        Ok(Self(url))
    }
    pub fn as_url(&self) -> &Url {
        &self.0
    }
}
impl FromStr for CloudTerrastodonAccountService {
    type Err = eyre::Report;
    fn from_str(value: &str) -> Result<Self> {
        Self::new(Url::parse(value)?)
    }
}
impl TryFrom<String> for CloudTerrastodonAccountService {
    type Error = eyre::Report;
    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}
impl From<&CloudTerrastodonAccountService> for String {
    fn from(value: &CloudTerrastodonAccountService) -> Self {
        value.0.to_string()
    }
}
impl fmt::Display for CloudTerrastodonAccountService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
cloud_terrastodon_registry::register_thing!(CloudTerrastodonAccountService);
