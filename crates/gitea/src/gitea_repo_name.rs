use crate::gitea_owner_name::validate_segment;
use compact_str::CompactString;
use facet::Facet;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Facet)]
#[facet(transparent)]
pub struct GiteaRepoName(CompactString);

impl GiteaRepoName {
    pub fn try_new(value: impl Into<CompactString>) -> eyre::Result<Self> {
        let value = value.into();
        validate_segment("repository name", &value)?;
        Ok(Self(value))
    }
}

impl Display for GiteaRepoName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for GiteaRepoName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for GiteaRepoName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for GiteaRepoName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

cloud_terrastodon_registry::register_thing!(GiteaRepoName);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoName);
