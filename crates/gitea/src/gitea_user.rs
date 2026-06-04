use crate::GiteaUserId;
use crate::GiteaUsername;
use chrono::DateTime;
use chrono::FixedOffset;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GiteaUser {
    pub id: GiteaUserId,
    pub login: GiteaUsername,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub html_url: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub last_login: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub created: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub active: bool,
}

impl Display for GiteaUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(full_name) = &self.full_name
            && !full_name.trim().is_empty()
        {
            write!(f, "{} ({full_name})", self.login)
        } else {
            self.login.fmt(f)
        }
    }
}
