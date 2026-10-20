use crate::domain::GroupRelationship;
use cloud_terrastodon::azure_types::EntraGroupDisplayName;
use cloud_terrastodon::azure_types::EntraPrincipalId;
use cloud_terrastodon::azure_types::UserPrincipalName;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

/// Reviewable drift or inventory limitations that do not by themselves block a plan.
#[derive(Debug, Clone, Eq, PartialEq, facet::Facet)]
#[repr(C)]
#[facet(tag = "code", rename_all = "camelCase")]
pub enum GroupPlanWarning {
    #[facet(rename_all = "camelCase")]
    GroupDisplayNameMismatch {
        expected: EntraGroupDisplayName,
        observed: EntraGroupDisplayName,
    },
    #[facet(rename_all = "camelCase")]
    IncompleteAdditiveInventory { relationship: GroupRelationship },
    #[facet(rename_all = "camelCase")]
    PrincipalUserPrincipalNameMismatch {
        principal_id: EntraPrincipalId,
        expected: UserPrincipalName,
        observed: UserPrincipalName,
    },
    #[facet(rename_all = "camelCase")]
    PrincipalUserPrincipalNameUnavailable {
        principal_id: EntraPrincipalId,
        expected: UserPrincipalName,
    },
    #[facet(rename_all = "camelCase")]
    PrincipalDisplayNameMismatch {
        principal_id: EntraPrincipalId,
        expected: String,
        observed: Option<String>,
    },
}

impl GroupPlanWarning {
    /// Compare structured facts without depending on human-facing wording.
    pub(crate) fn compare(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                Self::GroupDisplayNameMismatch { expected, observed },
                Self::GroupDisplayNameMismatch {
                    expected: other_expected,
                    observed: other_observed,
                },
            ) => (expected, observed).cmp(&(other_expected, other_observed)),
            (
                Self::IncompleteAdditiveInventory { relationship },
                Self::IncompleteAdditiveInventory {
                    relationship: other_relationship,
                },
            ) => relationship.as_str().cmp(other_relationship.as_str()),
            (
                Self::PrincipalUserPrincipalNameMismatch {
                    principal_id,
                    expected,
                    observed,
                },
                Self::PrincipalUserPrincipalNameMismatch {
                    principal_id: other_id,
                    expected: other_expected,
                    observed: other_observed,
                },
            ) => (principal_id.as_ref(), expected.as_str(), observed.as_str()).cmp(&(
                other_id.as_ref(),
                other_expected.as_str(),
                other_observed.as_str(),
            )),
            (
                Self::PrincipalUserPrincipalNameUnavailable {
                    principal_id,
                    expected,
                },
                Self::PrincipalUserPrincipalNameUnavailable {
                    principal_id: other_id,
                    expected: other_expected,
                },
            ) => (principal_id.as_ref(), expected.as_str())
                .cmp(&(other_id.as_ref(), other_expected.as_str())),
            (
                Self::PrincipalDisplayNameMismatch {
                    principal_id,
                    expected,
                    observed,
                },
                Self::PrincipalDisplayNameMismatch {
                    principal_id: other_id,
                    expected: other_expected,
                    observed: other_observed,
                },
            ) => (principal_id.as_ref(), expected, observed.as_deref()).cmp(&(
                other_id.as_ref(),
                other_expected,
                other_observed.as_deref(),
            )),
            _ => self.rank().cmp(&other.rank()),
        }
    }

    const fn rank(&self) -> u8 {
        match self {
            Self::GroupDisplayNameMismatch { .. } => 0,
            Self::IncompleteAdditiveInventory { .. } => 1,
            Self::PrincipalUserPrincipalNameMismatch { .. } => 2,
            Self::PrincipalUserPrincipalNameUnavailable { .. } => 3,
            Self::PrincipalDisplayNameMismatch { .. } => 4,
        }
    }
}

impl Display for GroupPlanWarning {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::GroupDisplayNameMismatch { expected, observed } => write!(
                f,
                "Group displayName annotation {:?} differs from the current name {:?}; no rename is proposed",
                expected.as_str(),
                observed.as_str(),
            ),
            Self::IncompleteAdditiveInventory { relationship } => write!(
                f,
                "The {relationship} inventory can omit objects; additions are proposals requiring a fresh existence check before any future apply"
            ),
            Self::PrincipalUserPrincipalNameMismatch {
                principal_id,
                expected,
                observed,
            } => write!(
                f,
                "Pinned principal {principal_id} has userPrincipalName {observed}; the document still says {expected}"
            ),
            Self::PrincipalUserPrincipalNameUnavailable {
                principal_id,
                expected,
            } => write!(
                f,
                "Pinned principal {principal_id} has no observed userPrincipalName; annotation {expected} cannot be confirmed"
            ),
            Self::PrincipalDisplayNameMismatch {
                principal_id,
                expected,
                observed,
            } => write!(
                f,
                "Principal {principal_id} displayName annotation {expected:?} differs from observed {observed:?}"
            ),
        }
    }
}
