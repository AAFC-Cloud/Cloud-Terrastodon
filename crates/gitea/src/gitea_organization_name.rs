use crate::gitea_owner_name::validate_segment;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use facet::Facet;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Facet)]
#[facet(transparent)]
pub struct GiteaOrganizationName(CompactString);

impl GiteaOrganizationName {
    pub fn try_new(value: impl Into<CompactString>) -> eyre::Result<Self> {
        let value = value.into();
        validate_segment("organization name", &value)?;
        Ok(Self(value))
    }
}

impl Display for GiteaOrganizationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for GiteaOrganizationName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for GiteaOrganizationName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for GiteaOrganizationName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

impl<'a> Arbitrary<'a> for GiteaOrganizationName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut value = String::arbitrary(u)?.replace('/', "");
        value = value.trim().to_string();
        if value.is_empty() {
            value.push('x');
        }
        GiteaOrganizationName::try_new(value).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}
cloud_terrastodon_registry::register_thing!(GiteaOrganizationName);
cloud_terrastodon_registry::register_arbitrary!(GiteaOrganizationName);
