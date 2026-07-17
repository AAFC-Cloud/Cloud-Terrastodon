use compact_str::CompactString;
use eyre::bail;
use std::ops::Deref;
use std::str::FromStr;

/// The Microsoft Graph resource URI used in Entra OAuth scope requests.
pub const MICROSOFT_GRAPH_SCOPE_PREFIX: &str = "https://graph.microsoft.com/";

/// A Microsoft Graph delegated permission claim value, such as `User.Read`.
///
/// This stores the bare permission value used by `oauth2PermissionGrant.scope`.
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, facet::Facet, arbitrary::Arbitrary,
)]
#[facet(transparent)]
pub struct MicrosoftGraphScopeClaim(CompactString);

impl MicrosoftGraphScopeClaim {
    pub fn try_new(value: impl Into<CompactString>) -> eyre::Result<Self> {
        let value = value.into();
        if value.is_empty() {
            bail!("Microsoft Graph scope claim cannot be empty");
        }
        if value.chars().any(char::is_whitespace) {
            bail!("Microsoft Graph scope claim cannot contain whitespace: {value:?}");
        }
        if value.starts_with(MICROSOFT_GRAPH_SCOPE_PREFIX) {
            bail!(
                "Microsoft Graph scope claim must be the bare permission value, not the resource URI: {value:?}"
            );
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for MicrosoftGraphScopeClaim {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for MicrosoftGraphScopeClaim {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for MicrosoftGraphScopeClaim {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<&String> for MicrosoftGraphScopeClaim {
    type Error = eyre::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_new(value.as_str())
    }
}

impl TryFrom<CompactString> for MicrosoftGraphScopeClaim {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<MicrosoftGraphScopeClaim> for CompactString {
    fn from(value: MicrosoftGraphScopeClaim) -> Self {
        value.0
    }
}

impl std::fmt::Display for MicrosoftGraphScopeClaim {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Deref for MicrosoftGraphScopeClaim {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

cloud_terrastodon_registry::register_thing!(MicrosoftGraphScopeClaim);

#[cfg(test)]
mod tests {
    use super::MicrosoftGraphScopeClaim;

    #[test]
    fn accepts_a_bare_graph_permission_value() -> eyre::Result<()> {
        let scope = MicrosoftGraphScopeClaim::try_new("User.Read")?;

        assert_eq!(scope.as_str(), "User.Read");
        assert_eq!(scope.to_string(), "User.Read");
        Ok(())
    }

    #[test]
    fn rejects_a_resource_qualified_scope() {
        assert!(
            MicrosoftGraphScopeClaim::try_new("https://graph.microsoft.com/User.Read").is_err()
        );
    }

    #[test]
    fn rejects_whitespace_in_scope_values() {
        assert!(MicrosoftGraphScopeClaim::try_new("User.Read Group.Read").is_err());
    }
}
