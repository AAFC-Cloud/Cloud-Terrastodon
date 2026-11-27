use crate::AuthSource;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use std::fmt;
use std::str::FromStr;

/// Concrete credential slot within a Cloud Terrastodon account. Unlike CLI
/// preferences, this cannot represent automatic selection or disabled auth.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Facet, Arbitrary)]
#[facet(proxy = String)]
#[repr(u8)]
pub enum CloudTerrastodonCredentialSource {
    AzureCli,
    Browser,
    WorkloadIdentity,
    PersonalAccessToken,
    Gitea,
    BmcHelix,
}

impl CloudTerrastodonCredentialSource {
    pub const fn auth_source(self) -> Option<AuthSource> {
        match self {
            Self::AzureCli => Some(AuthSource::AzureCli),
            Self::Browser => Some(AuthSource::Browser),
            Self::WorkloadIdentity => Some(AuthSource::WorkloadIdentity),
            Self::PersonalAccessToken => Some(AuthSource::PersonalAccessToken),
            Self::Gitea | Self::BmcHelix => None,
        }
    }
}

impl TryFrom<AuthSource> for CloudTerrastodonCredentialSource {
    type Error = eyre::Report;
    fn try_from(value: AuthSource) -> Result<Self> {
        match value {
            AuthSource::AzureCli => Ok(Self::AzureCli),
            AuthSource::Browser => Ok(Self::Browser),
            AuthSource::WorkloadIdentity => Ok(Self::WorkloadIdentity),
            AuthSource::PersonalAccessToken => Ok(Self::PersonalAccessToken),
            AuthSource::Auto | AuthSource::None => {
                eyre::bail!("a concrete credential source is required");
            }
        }
    }
}

impl fmt::Display for CloudTerrastodonCredentialSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gitea => formatter.write_str("gitea"),
            Self::BmcHelix => formatter.write_str("helix"),
            Self::AzureCli => AuthSource::AzureCli.fmt(formatter),
            Self::Browser => AuthSource::Browser.fmt(formatter),
            Self::WorkloadIdentity => AuthSource::WorkloadIdentity.fmt(formatter),
            Self::PersonalAccessToken => AuthSource::PersonalAccessToken.fmt(formatter),
        }
    }
}
impl FromStr for CloudTerrastodonCredentialSource {
    type Err = eyre::Report;
    fn from_str(value: &str) -> Result<Self> {
        if value.eq_ignore_ascii_case("gitea") {
            return Ok(Self::Gitea);
        }
        if value.eq_ignore_ascii_case("helix") {
            return Ok(Self::BmcHelix);
        }
        value.parse::<AuthSource>()?.try_into()
    }
}
impl TryFrom<String> for CloudTerrastodonCredentialSource {
    type Error = eyre::Report;
    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}
impl From<&CloudTerrastodonCredentialSource> for String {
    fn from(value: &CloudTerrastodonCredentialSource) -> Self {
        value.to_string()
    }
}
cloud_terrastodon_registry::register_thing!(CloudTerrastodonCredentialSource);
cloud_terrastodon_registry::register_arbitrary!(CloudTerrastodonCredentialSource);
