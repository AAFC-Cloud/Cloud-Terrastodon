use crate::GiteaOrganizationId;
use crate::GiteaOrganizationName;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GiteaOrganization {
    pub id: GiteaOrganizationId,
    pub username: GiteaOrganizationName,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub visibility: Option<String>,
}

impl Display for GiteaOrganization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(full_name) = &self.full_name
            && !full_name.trim().is_empty()
        {
            write!(f, "{} ({full_name})", self.username)
        } else {
            self.username.fmt(f)
        }
    }
}
