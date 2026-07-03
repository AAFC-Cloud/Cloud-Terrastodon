use facet::Facet;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Facet)]
#[facet(transparent)]
pub struct GiteaOrganizationId(u64);

impl GiteaOrganizationId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

impl Display for GiteaOrganizationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for GiteaOrganizationId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for GiteaOrganizationId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse()?))
    }
}

cloud_terrastodon_registry::register_thing!(GiteaOrganizationId);
cloud_terrastodon_registry::register_arbitrary!(GiteaOrganizationId);
