use eyre::bail;
use std::str::FromStr;

/// Standard OpenID Connect scope claim values understood by Microsoft Entra.
#[repr(C)]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, facet::Facet, arbitrary::Arbitrary,
)]
pub enum OpenIdConnectScopeClaim {
    OpenId,
    Profile,
    Email,
    Address,
    Phone,
    OfflineAccess,
}

impl OpenIdConnectScopeClaim {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenId => "openid",
            Self::Profile => "profile",
            Self::Email => "email",
            Self::Address => "address",
            Self::Phone => "phone",
            Self::OfflineAccess => "offline_access",
        }
    }
}

impl FromStr for OpenIdConnectScopeClaim {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "openid" => Ok(Self::OpenId),
            "profile" => Ok(Self::Profile),
            "email" => Ok(Self::Email),
            "address" => Ok(Self::Address),
            "phone" => Ok(Self::Phone),
            "offline_access" => Ok(Self::OfflineAccess),
            _ => bail!("unknown OpenID Connect scope claim: {value}"),
        }
    }
}

impl TryFrom<&str> for OpenIdConnectScopeClaim {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl std::fmt::Display for OpenIdConnectScopeClaim {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(OpenIdConnectScopeClaim);

#[cfg(test)]
mod tests {
    use super::OpenIdConnectScopeClaim;

    #[test]
    fn formats_offline_access() {
        assert_eq!(
            OpenIdConnectScopeClaim::OfflineAccess.to_string(),
            "offline_access"
        );
    }

    #[test]
    fn parses_standard_scopes() -> eyre::Result<()> {
        assert_eq!(
            "openid".parse::<OpenIdConnectScopeClaim>()?,
            OpenIdConnectScopeClaim::OpenId
        );
        assert_eq!(
            "offline_access".parse::<OpenIdConnectScopeClaim>()?,
            OpenIdConnectScopeClaim::OfflineAccess
        );
        Ok(())
    }
}
