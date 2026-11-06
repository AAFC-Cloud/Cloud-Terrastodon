use crate::EntraPrincipalId;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::str::FromStr;
use url::Url;

/// Microsoft Graph v1.0 directory-object reference used by group membership writes.
///
/// [Group member reference syntax](https://learn.microsoft.com/graph/api/group-post-members?view=graph-rest-1.0#request-body).
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(proxy = String)]
pub struct EntraDirectoryObjectUrl {
    pub id: EntraPrincipalId,
}

impl FromStr for EntraDirectoryObjectUrl {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        let url = Url::parse(value)?;
        ensure!(
            url.scheme() == "https"
                && url.host_str() == Some("graph.microsoft.com")
                && url.port().is_none()
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none(),
            "directory-object references require the Microsoft Graph HTTPS endpoint"
        );
        let segments = url
            .path()
            .trim_start_matches('/')
            .split('/')
            .collect::<Vec<_>>();
        match segments.as_slice() {
            ["v1.0", "directoryObjects", id] => Ok(Self { id: id.parse()? }),
            _ => {
                eyre::bail!("expected a Microsoft Graph v1.0 directory-object reference");
            }
        }
    }
}

impl TryFrom<String> for EntraDirectoryObjectUrl {
    type Error = eyre::Report;
    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl From<&EntraDirectoryObjectUrl> for String {
    fn from(value: &EntraDirectoryObjectUrl) -> Self {
        format!(
            "https://graph.microsoft.com/v1.0/directoryObjects/{}",
            value.id
        )
    }
}

cloud_terrastodon_registry::register_thing!(EntraDirectoryObjectUrl);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryObjectUrl);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_preserve_identity_and_reject_other_routes_or_authorities() {
        let value = "https://graph.microsoft.com/v1.0/directoryObjects/00000000-0000-0000-0000-000000000001";
        let reference: EntraDirectoryObjectUrl = value.parse().unwrap();
        assert_eq!(String::from(&reference), value);
        for invalid in [
            value.replace("graph.microsoft.com", "graph.microsoft.com.evil.test"),
            value.replace("directoryObjects", "users"),
            value.replace("https://", "https://user@"),
            format!("{value}?extra=1"),
            format!("{value}/$ref"),
        ] {
            assert!(
                invalid.parse::<EntraDirectoryObjectUrl>().is_err(),
                "{invalid}"
            );
        }
    }
}
