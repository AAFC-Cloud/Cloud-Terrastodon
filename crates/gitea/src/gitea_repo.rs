use crate::GiteaRepoFullName;
use crate::GiteaRepoId;
use crate::GiteaRepoName;
use crate::GiteaUser;
use chrono::DateTime;
use chrono::FixedOffset;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GiteaRepo {
    pub id: GiteaRepoId,
    pub name: GiteaRepoName,
    pub full_name: GiteaRepoFullName,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub private: bool,
    pub owner: GiteaUser,
    #[serde(default)]
    pub html_url: Option<String>,
    #[serde(default)]
    pub ssh_url: Option<String>,
    #[serde(default)]
    pub clone_url: Option<String>,
    #[serde(default)]
    pub default_branch: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub archived: bool,
}

impl Display for GiteaRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.full_name.fmt(f)
    }
}
