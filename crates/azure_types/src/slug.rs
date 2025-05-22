use compact_str::CompactString;
use std::str::FromStr;

pub trait Slug: Sized + FromStr {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self>;
    fn validate_slug(&self) -> eyre::Result<()>;
}
impl Slug for CompactString {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        Ok(name.into())
    }
    fn validate_slug(&self) -> eyre::Result<()> {
        Ok(())
    }
}

/*
TODO: remove try_new in favour of using TryFrom trait
pub trait Slug: Sized + FromStr + TryFrom<CompactString> {
    fn validate_slug(&self) -> eyre::Result<()>;
}
impl Slug for CompactString {
    fn validate_slug(&self) -> eyre::Result<()> {
        Ok(())
    }
}

*/

pub trait HasSlug {
    /// Associated type with a default
    type Name: Slug = CompactString;

    /// Gets a reference to the name
    fn name(&self) -> &Self::Name;
}
