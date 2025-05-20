use arbitrary::Arbitrary;
use compact_str::CompactString;
use std::str::FromStr;

use crate::scopes::strip_prefix_case_insensitive;

pub trait Segment: Sized + FromStr + Arbitrary<'static> {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self>;
    fn validate_segment(&self) -> eyre::Result<()>;
}
impl Segment for CompactString {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        Ok(name.into())
    }
    fn validate_segment(&self) -> eyre::Result<()> {
        Ok(())
    }
}
pub trait HasSegment<T:Segment> {
    fn get_segment(&self) -> &T;
}

// ======================


// ======================

pub struct SlashSubscriptionSlashLiteral;
impl Segment for SlashSubscriptionSlashLiteral {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        Ok("/subscription/")
    }

    fn validate_segment(&self) -> eyre::Result<()> {
        Ok(())
    }
}
impl FromStr for SlashSubscriptionSlashLiteral {
    type Err=eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = strip_prefix_case_insensitive(s, "/subscription/");
        if !s.is_empty() {
            Err(eyre::Error::msg("Invalid segment"))
        } else {
            Ok("/subscription/")
        }
    }
}
impl<'a> Arbitrary<'a> for SlashSubscriptionSlashLiteral {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok("/subscription/")
    }
}