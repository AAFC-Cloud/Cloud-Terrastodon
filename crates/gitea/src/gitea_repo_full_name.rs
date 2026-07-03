use crate::GiteaOwnerName;
use crate::GiteaRepoName;
use facet::Facet;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Facet)]
#[facet(opaque, proxy = GiteaRepoFullNameProxy)]
pub struct GiteaRepoFullName {
    pub owner: GiteaOwnerName,
    pub repo_name: GiteaRepoName,
}

#[derive(Debug, Clone, Eq, PartialEq, Facet)]
#[facet(transparent)]
pub struct GiteaRepoFullNameProxy(String);

impl GiteaRepoFullName {
    pub fn try_new(owner: GiteaOwnerName, repo_name: GiteaRepoName) -> eyre::Result<Self> {
        Ok(Self { owner, repo_name })
    }
}

impl Display for GiteaRepoFullName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.owner, self.repo_name)
    }
}

impl FromStr for GiteaRepoFullName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((owner, repo_name)) = s.split_once('/') else {
            eyre::bail!("Repository full name must be in the format owner/repo");
        };
        if repo_name.contains('/') {
            eyre::bail!("Repository full name must contain exactly one '/'");
        }
        Self::try_new(
            owner.parse::<GiteaOwnerName>()?,
            repo_name.parse::<GiteaRepoName>()?,
        )
    }
}

impl TryFrom<GiteaRepoFullNameProxy> for GiteaRepoFullName {
    type Error = eyre::Error;

    fn try_from(value: GiteaRepoFullNameProxy) -> Result<Self, Self::Error> {
        Self::from_str(&value.0)
    }
}

impl From<&GiteaRepoFullName> for GiteaRepoFullNameProxy {
    fn from(value: &GiteaRepoFullName) -> Self {
        Self(value.to_string())
    }
}

cloud_terrastodon_registry::register_thing!(GiteaRepoFullName);
