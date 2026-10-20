use std::fmt::Display;
use std::fmt::Formatter;

/// A relationship governed by this application's reconciliation policy.
///
/// Members and owners have different eligibility rules and independently scoped
/// diagnostics. SDK reads use their corresponding member or owner request types.
#[derive(Debug, Clone, Copy, Eq, PartialEq, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum GroupRelationship {
    Members,
    Owners,
}

impl GroupRelationship {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Members => "members",
            Self::Owners => "owners",
        }
    }
}

impl Display for GroupRelationship {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
