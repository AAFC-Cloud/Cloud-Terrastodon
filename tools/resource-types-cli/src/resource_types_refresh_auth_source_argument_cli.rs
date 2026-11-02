use cloud_terrastodon_credentials::AuthSource;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::str::FromStr;

/// Explicit bearer credential selection accepted by the metadata refresh command.
///
/// Validation rejects automatic selection, disabled authentication, and PATs
/// before the command resolves any credentials or reads authentication defaults.
#[derive(Debug, Clone, Copy, Facet)]
#[facet(proxy = String)]
pub struct ResourceTypesRefreshAuthSourceArgument(AuthSource);

impl TryFrom<AuthSource> for ResourceTypesRefreshAuthSourceArgument {
    type Error = eyre::Report;

    fn try_from(source: AuthSource) -> Result<Self> {
        ensure!(
            matches!(
                source,
                AuthSource::AzureCli | AuthSource::Browser | AuthSource::WorkloadIdentity
            ),
            "resource-type refresh requires explicit azure-cli, browser, or workload-identity authentication"
        );
        Ok(Self(source))
    }
}

impl From<ResourceTypesRefreshAuthSourceArgument> for AuthSource {
    fn from(source: ResourceTypesRefreshAuthSourceArgument) -> Self {
        source.0
    }
}

impl FromStr for ResourceTypesRefreshAuthSourceArgument {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_from(AuthSource::from_str(value)?)
    }
}

impl TryFrom<String> for ResourceTypesRefreshAuthSourceArgument {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl From<&ResourceTypesRefreshAuthSourceArgument> for String {
    fn from(source: &ResourceTypesRefreshAuthSourceArgument) -> Self {
        source.0.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::ResourceTypesRefreshAuthSourceArgument;
    use cloud_terrastodon_credentials::AuthSource;

    #[test]
    fn unsuitable_authentication_is_rejected_without_resolving_runtime_state() {
        for source in [
            AuthSource::Auto,
            AuthSource::None,
            AuthSource::PersonalAccessToken,
        ] {
            assert!(ResourceTypesRefreshAuthSourceArgument::try_from(source).is_err());
            assert!(
                source
                    .to_string()
                    .parse::<ResourceTypesRefreshAuthSourceArgument>()
                    .is_err()
            );
        }
        for source in [
            AuthSource::AzureCli,
            AuthSource::Browser,
            AuthSource::WorkloadIdentity,
        ] {
            let selection = ResourceTypesRefreshAuthSourceArgument::try_from(source).unwrap();
            assert_eq!(AuthSource::from(selection), source);
        }
    }
}
