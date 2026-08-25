use eyre::bail;
use std::fmt::Display;
use std::str::FromStr;

/// Selects the credential source used by Azure resource requests.
///
/// `Auto` prefers a complete workload-identity environment. In a headless
/// context it retains Azure CLI as a disabled compatibility source so
/// non-Azure commands can still run; the credential boundary reports the
/// actionable authentication error if an Azure token is requested.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, arbitrary::Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(u8)]
pub enum AuthSource {
    #[default]
    Auto,
    WorkloadIdentity,
    Browser,
    AzureCli,
    PersonalAccessToken,
}

impl Display for AuthSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Auto => "auto",
            Self::WorkloadIdentity => "workload-identity",
            Self::Browser => "browser",
            Self::AzureCli => "azure-cli",
            Self::PersonalAccessToken => "pat",
        })
    }
}

impl From<&AuthSource> for String {
    fn from(value: &AuthSource) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for AuthSource {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl FromStr for AuthSource {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "workload-identity" | "workload_identity" | "wif" => Ok(Self::WorkloadIdentity),
            "browser" | "delegated" => Ok(Self::Browser),
            "azure-cli" | "azure_cli" | "cli" => Ok(Self::AzureCli),
            "pat" | "personal-access-token" | "personal_access_token" => {
                Ok(Self::PersonalAccessToken)
            }
            _ => bail!(
                "unsupported authentication source {value:?}; expected auto, workload-identity, browser, azure-cli, or pat"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_sources_and_aliases() -> eyre::Result<()> {
        assert_eq!("auto".parse::<AuthSource>()?, AuthSource::Auto);
        assert_eq!(
            "workload_identity".parse::<AuthSource>()?,
            AuthSource::WorkloadIdentity
        );
        assert_eq!("delegated".parse::<AuthSource>()?, AuthSource::Browser);
        assert_eq!("cli".parse::<AuthSource>()?, AuthSource::AzureCli);
        assert_eq!(
            "pat".parse::<AuthSource>()?,
            AuthSource::PersonalAccessToken
        );
        assert!("secret".parse::<AuthSource>().is_err());
        Ok(())
    }

    #[test]
    fn displays_stable_cli_values() {
        assert_eq!(AuthSource::Auto.to_string(), "auto");
        assert_eq!(
            AuthSource::WorkloadIdentity.to_string(),
            "workload-identity"
        );
        assert_eq!(AuthSource::Browser.to_string(), "browser");
        assert_eq!(AuthSource::AzureCli.to_string(), "azure-cli");
        assert_eq!(AuthSource::PersonalAccessToken.to_string(), "pat");
    }
}
