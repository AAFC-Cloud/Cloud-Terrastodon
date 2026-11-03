use crate::AuthSource;
use arbitrary::Arbitrary;
use eyre::Report;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::fmt;
use std::str::FromStr;

/// A resolved source for tenant-bound Microsoft Entra bearer authentication.
///
/// This request vocabulary excludes automatic selection, disabled authentication,
/// and personal access tokens. It identifies acquisition policy without carrying
/// a token or consulting the environment.
/// See [Microsoft Entra authentication for Azure DevOps](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/entra-oauth?view=azure-devops).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(u8)]
pub enum AzureBearerAuthSource {
    AzureCli,
    WorkloadIdentity,
    Browser,
}

impl TryFrom<AuthSource> for AzureBearerAuthSource {
    type Error = Report;

    fn try_from(source: AuthSource) -> Result<Self> {
        match source {
            AuthSource::AzureCli => Ok(Self::AzureCli),
            AuthSource::WorkloadIdentity => Ok(Self::WorkloadIdentity),
            AuthSource::Browser => Ok(Self::Browser),
            AuthSource::Auto | AuthSource::None | AuthSource::PersonalAccessToken => {
                bail!("authentication source {source} is not a resolved Azure bearer source");
            }
        }
    }
}

impl From<AzureBearerAuthSource> for AuthSource {
    fn from(source: AzureBearerAuthSource) -> Self {
        match source {
            AzureBearerAuthSource::AzureCli => Self::AzureCli,
            AzureBearerAuthSource::WorkloadIdentity => Self::WorkloadIdentity,
            AzureBearerAuthSource::Browser => Self::Browser,
        }
    }
}

impl fmt::Display for AzureBearerAuthSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&AuthSource::from(*self), formatter)
    }
}

impl FromStr for AzureBearerAuthSource {
    type Err = Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_from(value.parse::<AuthSource>()?)
    }
}

impl TryFrom<String> for AzureBearerAuthSource {
    type Error = Report;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl From<&AzureBearerAuthSource> for String {
    fn from(value: &AzureBearerAuthSource) -> Self {
        value.to_string()
    }
}

cloud_terrastodon_registry::register_thing!(AzureBearerAuthSource);
cloud_terrastodon_registry::register_arbitrary!(AzureBearerAuthSource);
cloud_terrastodon_registry::register_try_from!(AuthSource => AzureBearerAuthSource);
cloud_terrastodon_registry::register_from!(AzureBearerAuthSource => AuthSource);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_source_rejects_unresolved_disabled_and_pat_preferences() {
        for source in [
            AuthSource::Auto,
            AuthSource::None,
            AuthSource::PersonalAccessToken,
        ] {
            assert!(AzureBearerAuthSource::try_from(source).is_err());
            assert!(source.to_string().parse::<AzureBearerAuthSource>().is_err());
        }
        let selected = "cli".parse::<AzureBearerAuthSource>().unwrap();
        assert_eq!(AuthSource::from(selected), AuthSource::AzureCli);
        assert_eq!(selected.to_string(), "azure-cli");
    }
}
