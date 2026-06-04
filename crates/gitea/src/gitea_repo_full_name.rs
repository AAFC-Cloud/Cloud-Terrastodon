use crate::GiteaOwnerName;
use crate::GiteaRepoName;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GiteaRepoFullName {
    pub owner: GiteaOwnerName,
    pub repo_name: GiteaRepoName,
}

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

impl Serialize for GiteaRepoFullName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for GiteaRepoFullName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}
