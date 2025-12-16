use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use eyre::bail;
use std::hash::Hash;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialOrd, Ord)]
pub struct ComputePublisherName(CompactString);
impl PartialEq for ComputePublisherName {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}
impl core::fmt::Display for ComputePublisherName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Hash for ComputePublisherName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state);
    }
}
impl Slug for ComputePublisherName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        validate_compute_publisher_name(&inner)?;
        Ok(Self(inner))
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        validate_compute_publisher_name(&self.0)?;
        Ok(())
    }
}
impl FromStr for ComputePublisherName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ComputePublisherName::try_new(s)
    }
}

fn validate_compute_publisher_name(value: &str) -> eyre::Result<()> {
    if !(1..=256).contains(&value.len()) {
        bail!(
            "Compute Publisher Name '{}' must be between 1 and 256 characters long",
            value
        )
    }
    Ok(())
}

impl Deref for ComputePublisherName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<str> for ComputePublisherName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'a> Arbitrary<'a> for ComputePublisherName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut s = CompactString::arbitrary(u)?;
        if s.len() > 256 {
            s.truncate(256);
        } else if s.is_empty() {
            s.push('a');
        }
        Ok(ComputePublisherName::try_new(s).map_err(|_| arbitrary::Error::IncorrectFormat)?)
    }
}
