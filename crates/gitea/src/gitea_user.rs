use crate::GiteaUserId;
use crate::GiteaUsername;
use chrono::DateTime;
use chrono::FixedOffset;
use facet::Facet;
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Facet)]
pub struct GiteaUser {
    pub id: GiteaUserId,
    pub login: GiteaUsername,
    #[facet(default)]
    pub full_name: Option<String>,
    #[facet(default)]
    pub email: Option<String>,
    #[facet(default)]
    pub avatar_url: Option<String>,
    #[facet(default)]
    pub html_url: Option<String>,
    #[facet(default)]
    pub language: Option<String>,
    #[facet(default)]
    pub last_login: Option<DateTime<FixedOffset>>,
    #[facet(default)]
    pub created: Option<DateTime<FixedOffset>>,
    #[facet(default)]
    pub visibility: Option<String>,
    #[facet(default)]
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

cloud_terrastodon_registry::register_thing!(GiteaUser);

