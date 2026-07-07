use arbitrary::Arbitrary;
use blake3::Hash;
use compact_str::CompactString;
use eyre::bail;
use facet::Facet;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Facet)]
#[facet(transparent)]
pub struct GiteaInstanceUrl(CompactString);

impl GiteaInstanceUrl {
    pub fn try_new(value: impl Into<CompactString>) -> eyre::Result<Self> {
        let value = value.into();
        let normalized = normalize_instance_url(value.as_str())?;
        Ok(Self(CompactString::from(normalized)))
    }

    pub fn api_url(&self, endpoint: &str) -> String {
        if endpoint.starts_with("https://") || endpoint.starts_with("http://") {
            endpoint.to_string()
        } else if endpoint.starts_with("/api/") {
            format!("{}{}", self.0, endpoint)
        } else if endpoint.starts_with('/') {
            format!("{}/api/v1{}", self.0, endpoint)
        } else {
            format!("{}/api/v1/{}", self.0, endpoint)
        }
    }

    pub fn storage_key(&self) -> String {
        let digest = Hash::to_hex(&blake3::hash(self.0.as_bytes()));
        let readable = self
            .0
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() {
                    ch.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string();
        let readable = if readable.is_empty() {
            "gitea".to_string()
        } else {
            readable
        };
        format!("{readable}-{}", &digest[..8])
    }
}

fn normalize_instance_url(value: &str) -> eyre::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("Gitea instance URL cannot be empty");
    }
    let value = value.trim_end_matches('/');
    let Some((scheme, remainder)) = value.split_once("://") else {
        bail!("Gitea instance URL must start with http:// or https://");
    };
    if !matches!(scheme, "http" | "https") {
        bail!("Gitea instance URL must use http or https");
    }
    if remainder.is_empty() {
        bail!("Gitea instance URL is missing a host");
    }
    if remainder.contains('?') || remainder.contains('#') {
        bail!("Gitea instance URL must not contain a query string or fragment");
    }
    if let Some((authority, path)) = remainder.split_once('/') {
        if authority.is_empty() {
            bail!("Gitea instance URL is missing a host");
        }
        if !path.is_empty() {
            bail!("Gitea instance URL must not contain a path");
        }
    }
    Ok(value.to_string())
}

impl Display for GiteaInstanceUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for GiteaInstanceUrl {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for GiteaInstanceUrl {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for GiteaInstanceUrl {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

impl TryFrom<&str> for GiteaInstanceUrl {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for GiteaInstanceUrl {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<&GiteaInstanceUrl> for String {
    fn from(value: &GiteaInstanceUrl) -> Self {
        value.to_string()
    }
}
impl<'a> Arbitrary<'a> for GiteaInstanceUrl {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut host = String::arbitrary(u)?
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
            .collect::<String>();
        if host.is_empty() {
            host.push_str("gitea");
        }
        GiteaInstanceUrl::try_new(format!("https://{host}.example.com"))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}
cloud_terrastodon_registry::register_thing!(GiteaInstanceUrl);
cloud_terrastodon_registry::register_arbitrary!(GiteaInstanceUrl);

#[cfg(test)]
mod tests {
    use super::GiteaInstanceUrl;

    #[test]
    fn it_normalizes_trailing_slashes() -> eyre::Result<()> {
        let url = GiteaInstanceUrl::try_new("https://gitea.example.com/")?;
        assert_eq!(url.to_string(), "https://gitea.example.com");
        Ok(())
    }

    #[test]
    fn it_builds_api_urls() -> eyre::Result<()> {
        let url = GiteaInstanceUrl::try_new("https://gitea.example.com")?;
        assert_eq!(
            url.api_url("/orgs"),
            "https://gitea.example.com/api/v1/orgs"
        );
        assert_eq!(
            url.api_url("/api/v1/orgs"),
            "https://gitea.example.com/api/v1/orgs"
        );
        Ok(())
    }

    #[test]
    fn it_rejects_paths() {
        assert!(GiteaInstanceUrl::try_new("https://gitea.example.com/foo").is_err());
    }
}
