use arbitrary::Arbitrary;
use crate::GiteaOrganizationId;
use crate::GiteaOrganizationName;
use facet::Facet;
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, Facet)]
pub struct GiteaOrganization {
    pub id: GiteaOrganizationId,
    pub username: GiteaOrganizationName,
    #[facet(default)]
    pub full_name: Option<String>,
    #[facet(default)]
    pub description: Option<String>,
    #[facet(default)]
    pub avatar_url: Option<String>,
    #[facet(default)]
    pub website: Option<String>,
    #[facet(default)]
    pub location: Option<String>,
    #[facet(default)]
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

cloud_terrastodon_registry::register_thing!(GiteaOrganization);
cloud_terrastodon_registry::register_arbitrary!(GiteaOrganization);

