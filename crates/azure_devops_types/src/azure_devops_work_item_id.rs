use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::str::FromStr;

/// Organization-scoped numeric work item ID (saved query IDs are UUIDs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = i32)]
pub struct AzureDevOpsWorkItemId(i32);

impl AzureDevOpsWorkItemId {
    pub fn new(value: i32) -> Result<Self> {
        ensure!(value > 0, "Work item IDs must be positive integers");
        Ok(Self(value))
    }

    pub fn get(self) -> i32 {
        self.0
    }
}

impl TryFrom<i32> for AzureDevOpsWorkItemId {
    type Error = eyre::Report;
    fn try_from(value: i32) -> Result<Self> {
        Self::new(value)
    }
}

impl From<&AzureDevOpsWorkItemId> for i32 {
    fn from(value: &AzureDevOpsWorkItemId) -> Self {
        value.0
    }
}

impl FromStr for AzureDevOpsWorkItemId {
    type Err = eyre::Report;
    fn from_str(value: &str) -> Result<Self> {
        Self::new(value.parse()?)
    }
}

impl std::fmt::Display for AzureDevOpsWorkItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemId {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(u.int_in_range(1..=i32::MAX)?))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemId);
