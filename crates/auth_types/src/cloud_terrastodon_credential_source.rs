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
}

impl CloudTerrastodonCredentialSource {
    pub const fn auth_source(self) -> Option<AuthSource> {
        match self {
            Self::AzureCli => Some(AuthSource::AzureCli),
            Self::Browser => Some(AuthSource::Browser),
            Self::WorkloadIdentity => Some(AuthSource::WorkloadIdentity),
            Self::PersonalAccessToken => Some(AuthSource::PersonalAccessToken),
            Self::Gitea => None,
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
        match self.auth_source() {
            Some(source) => source.fmt(formatter),
            None => formatter.write_str("gitea"),
        }
    }
}
impl FromStr for CloudTerrastodonCredentialSource {
    type Err = eyre::Report;
    fn from_str(value: &str) -> Result<Self> {
        if value.eq_ignore_ascii_case("gitea") {
            return Ok(Self::Gitea);
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
