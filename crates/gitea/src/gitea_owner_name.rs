use arbitrary::Arbitrary;
use compact_str::CompactString;
use eyre::bail;
use facet::Facet;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Facet)]
#[facet(transparent)]
pub struct GiteaOwnerName(CompactString);

impl GiteaOwnerName {
    pub fn try_new(value: impl Into<CompactString>) -> eyre::Result<Self> {
        let value = value.into();
        validate_segment("owner", &value)?;
        Ok(Self(value))
    }
}

pub(crate) fn validate_segment(kind: &str, value: &str) -> eyre::Result<()> {
    let value = value.trim();
    if value.is_empty() {
        bail!("{kind} cannot be empty");
    }
    if value.contains('/') {
        bail!("{kind} cannot contain '/'");
    }
    Ok(())
}

impl Display for GiteaOwnerName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for GiteaOwnerName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for GiteaOwnerName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for GiteaOwnerName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

impl<'a> Arbitrary<'a> for GiteaOwnerName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut value = String::arbitrary(u)?.replace('/', "");
        value = value.trim().to_string();
        if value.is_empty() {
            value.push('x');
        }
        GiteaOwnerName::try_new(value).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}
cloud_terrastodon_registry::register_thing!(GiteaOwnerName);
cloud_terrastodon_registry::register_arbitrary!(GiteaOwnerName);

