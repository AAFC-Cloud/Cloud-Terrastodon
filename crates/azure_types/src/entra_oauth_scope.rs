use std::collections::HashSet;
use std::str::FromStr;

use crate::EntraOAuthScopeClaim;

/// The space-delimited collection represented by an Entra OAuth `scope`
/// parameter or an `oauth2PermissionGrant.scope` property.
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, arbitrary::Arbitrary, facet::Facet,
)]
#[facet(json::proxy = String)]
pub struct EntraOAuthScope(Vec<EntraOAuthScopeClaim>);

crate::impl_facet_string_proxy!(EntraOAuthScope, value => value.to_string());

impl EntraOAuthScope {
    pub fn try_new(
        raw_claims: impl IntoIterator<Item = impl Into<EntraOAuthScopeClaim>>,
    ) -> eyre::Result<Self> {
        let mut seen = HashSet::new();
        let mut claims = Vec::new();
        for claim in raw_claims {
            let claim = claim.into();
            if seen.insert(claim.clone()) {
                claims.push(claim);
            }
        }
        if claims.is_empty() {
            eyre::bail!("Entra OAuth scope cannot be empty");
        }
        Ok(Self(claims))
    }

    pub fn as_claims(&self) -> &[EntraOAuthScopeClaim] {
        &self.0
    }

    /// Return the space-delimited value sent to an Entra OAuth endpoint or
    /// written to `oauth2PermissionGrant.scope`.
    pub fn as_oauth_scope(&self) -> String {
        self.to_string()
    }
}

impl FromStr for EntraOAuthScope {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let claims = value
            .split_ascii_whitespace()
            .map(EntraOAuthScopeClaim::from_str)
            .collect::<Result<Vec<_>, _>>()?;
        Self::try_new(claims)
    }
}

impl TryFrom<&str> for EntraOAuthScope {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl std::fmt::Display for EntraOAuthScope {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut claims = self.0.iter();
        if let Some(claim) = claims.next() {
            claim.fmt(formatter)?;
        }
        for claim in claims {
            formatter.write_str(" ")?;
            claim.fmt(formatter)?;
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(EntraOAuthScope);

#[cfg(test)]
mod tests {
    use super::EntraOAuthScope;
    use super::EntraOAuthScopeClaim;
    use crate::MicrosoftGraphScopeClaim;
    use crate::OpenIdConnectScopeClaim;

    #[test]
    fn joins_bare_claim_values_with_spaces() -> eyre::Result<()> {
        let scope = EntraOAuthScope::try_new([
            EntraOAuthScopeClaim::from(OpenIdConnectScopeClaim::OpenId),
            EntraOAuthScopeClaim::from(MicrosoftGraphScopeClaim::try_new("User.Read")?),
        ])?;

        assert_eq!(scope.as_oauth_scope(), "openid User.Read");
        Ok(())
    }

    #[test]
    fn deduplicates_claim_values_while_preserving_order() -> eyre::Result<()> {
        let scope = "User.Read User.Read openid".parse::<EntraOAuthScope>()?;

        assert_eq!(scope.to_string(), "User.Read openid");
        Ok(())
    }

    #[test]
    fn parses_resource_qualified_graph_claims() -> eyre::Result<()> {
        let scope = "https://graph.microsoft.com/User.Read".parse::<EntraOAuthScope>()?;

        assert_eq!(scope.to_string(), "User.Read");
        Ok(())
    }

    #[test]
    fn serializes_as_the_graph_scope_string() -> eyre::Result<()> {
        let scope = "openid User.Read".parse::<EntraOAuthScope>()?;

        assert_eq!(facet_json::to_string(&scope)?, "\"openid User.Read\"");
        let reparsed = facet_json::from_str::<EntraOAuthScope>("\"openid User.Read\"")?;
        assert_eq!(scope, reparsed);
        Ok(())
    }
}
