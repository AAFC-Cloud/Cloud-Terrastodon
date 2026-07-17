use std::str::FromStr;

use crate::{MICROSOFT_GRAPH_SCOPE_PREFIX, MicrosoftGraphScopeClaim, OpenIdConnectScopeClaim};


/// One claim value in an Entra OAuth `scope` parameter.
#[repr(C)]
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, facet::Facet, arbitrary::Arbitrary,
)]
pub enum EntraOAuthScopeClaim {
    OpenIdConnectScopeClaim(OpenIdConnectScopeClaim),
    MicrosoftGraphScopeClaim(MicrosoftGraphScopeClaim),
}

impl From<OpenIdConnectScopeClaim> for EntraOAuthScopeClaim {
    fn from(value: OpenIdConnectScopeClaim) -> Self {
        Self::OpenIdConnectScopeClaim(value)
    }
}

impl From<MicrosoftGraphScopeClaim> for EntraOAuthScopeClaim {
    fn from(value: MicrosoftGraphScopeClaim) -> Self {
        Self::MicrosoftGraphScopeClaim(value)
    }
}

impl FromStr for EntraOAuthScopeClaim {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(scope) = OpenIdConnectScopeClaim::from_str(value) {
            return Ok(Self::OpenIdConnectScopeClaim(scope));
        }

        let graph_value = value
            .strip_prefix(MICROSOFT_GRAPH_SCOPE_PREFIX)
            .unwrap_or(value);
        Ok(Self::MicrosoftGraphScopeClaim(
            MicrosoftGraphScopeClaim::try_new(graph_value)?,
        ))
    }
}

impl TryFrom<&str> for EntraOAuthScopeClaim {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl std::fmt::Display for EntraOAuthScopeClaim {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenIdConnectScopeClaim(scope) => scope.fmt(formatter),
            Self::MicrosoftGraphScopeClaim(scope) => scope.fmt(formatter),
        }
    }
}
cloud_terrastodon_registry::register_thing!(EntraOAuthScopeClaim);
