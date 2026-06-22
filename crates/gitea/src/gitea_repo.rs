use crate::GiteaRepoFullName;
use crate::GiteaRepoId;
use crate::GiteaRepoName;
use crate::GiteaUser;
use chrono::DateTime;
use chrono::FixedOffset;
use facet::Facet;
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Facet)]
pub struct GiteaRepo {
    pub id: GiteaRepoId,
    pub name: GiteaRepoName,
    pub full_name: GiteaRepoFullName,
    #[facet(default)]
    pub description: Option<String>,
    #[facet(default)]
    pub private: bool,
    pub owner: GiteaUser,
    #[facet(default)]
    pub html_url: Option<String>,
    #[facet(default)]
    pub ssh_url: Option<String>,
    #[facet(default)]
    pub clone_url: Option<String>,
    #[facet(default)]
    pub default_branch: Option<String>,
    #[facet(default)]
    pub created_at: Option<DateTime<FixedOffset>>,
    #[facet(default)]
    pub updated_at: Option<DateTime<FixedOffset>>,
    #[facet(default)]
    pub topics: Vec<String>,
    #[facet(default)]
    pub archived: bool,
}

impl Display for GiteaRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.full_name.fmt(f)
    }
}
