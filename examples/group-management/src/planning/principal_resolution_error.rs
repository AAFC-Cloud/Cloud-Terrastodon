use cloud_terrastodon::azure_types::AzureTenantId;
use cloud_terrastodon::azure_types::EntraDirectoryObjectType;
use cloud_terrastodon::azure_types::EntraPrincipalId;
use cloud_terrastodon::azure_types::UserPrincipalName;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

/// A missing or inconsistent identity observation that prevents principal resolution.
#[derive(Debug, Clone, Eq, PartialEq, facet::Facet)]
#[repr(C)]
#[facet(tag = "code", rename_all = "camelCase")]
pub enum PrincipalResolutionError {
    #[facet(rename_all = "camelCase")]
    LookupNotCompleted {
        tenant_id: AzureTenantId,
    },
    #[facet(rename_all = "camelCase")]
    DuplicateLookupRecords {
        tenant_id: AzureTenantId,
    },
    #[facet(rename_all = "camelCase")]
    UpnReturnedNonUser {
        upn: UserPrincipalName,
        actual_kind: EntraDirectoryObjectType,
    },
    #[facet(rename_all = "camelCase")]
    UpnMismatch {
        upn: UserPrincipalName,
        principal_id: EntraPrincipalId,
        observed: Option<UserPrincipalName>,
    },
    #[facet(rename_all = "camelCase")]
    PinnedPrincipalNotFound {
        tenant_id: AzureTenantId,
        principal_id: EntraPrincipalId,
    },
    #[facet(rename_all = "camelCase")]
    PinnedIdMismatch {
        expected: EntraPrincipalId,
        observed: EntraPrincipalId,
    },
    #[facet(rename_all = "camelCase")]
    PinnedPrincipalNotUser {
        principal_id: EntraPrincipalId,
        upn: UserPrincipalName,
        actual_kind: EntraDirectoryObjectType,
    },
    #[facet(rename_all = "camelCase")]
    UpnIdMismatch {
        upn: UserPrincipalName,
        pinned_id: EntraPrincipalId,
        observed_id: EntraPrincipalId,
    },
    #[facet(rename_all = "camelCase")]
    ConflictingUpnLookup {
        principal_id: EntraPrincipalId,
        upn: UserPrincipalName,
    },
    MissingIdentity,
    #[facet(rename_all = "camelCase")]
    UpnNotFound {
        tenant_id: AzureTenantId,
        upn: UserPrincipalName,
    },
    NilPrincipalId,
}

impl PrincipalResolutionError {
    /// Compare exact identity facts, retaining absent annotations and UPN spelling.
    pub(crate) fn compare(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                Self::LookupNotCompleted { tenant_id },
                Self::LookupNotCompleted {
                    tenant_id: other_tenant,
                },
            )
            | (
                Self::DuplicateLookupRecords { tenant_id },
                Self::DuplicateLookupRecords {
                    tenant_id: other_tenant,
                },
            ) => tenant_id.cmp(other_tenant),
            (
                Self::UpnReturnedNonUser { upn, actual_kind },
                Self::UpnReturnedNonUser {
                    upn: other_upn,
                    actual_kind: other_kind,
                },
            ) => (upn.as_str(), actual_kind.graph_type())
                .cmp(&(other_upn.as_str(), other_kind.graph_type())),
            (
                Self::UpnMismatch {
                    upn,
                    principal_id,
                    observed,
                },
                Self::UpnMismatch {
                    upn: other_upn,
                    principal_id: other_id,
                    observed: other_observed,
                },
            ) => (
                upn.as_str(),
                principal_id.as_ref(),
                observed.as_ref().map(UserPrincipalName::as_str),
            )
                .cmp(&(
                    other_upn.as_str(),
                    other_id.as_ref(),
                    other_observed.as_ref().map(UserPrincipalName::as_str),
                )),
            (
                Self::PinnedPrincipalNotFound {
                    tenant_id,
                    principal_id,
                },
                Self::PinnedPrincipalNotFound {
                    tenant_id: other_tenant,
                    principal_id: other_id,
                },
            ) => (tenant_id, principal_id.as_ref()).cmp(&(other_tenant, other_id.as_ref())),
            (
                Self::PinnedIdMismatch { expected, observed },
                Self::PinnedIdMismatch {
                    expected: other_expected,
                    observed: other_observed,
                },
            ) => (expected.as_ref(), observed.as_ref())
                .cmp(&(other_expected.as_ref(), other_observed.as_ref())),
            (
                Self::PinnedPrincipalNotUser {
                    principal_id,
                    upn,
                    actual_kind,
                },
                Self::PinnedPrincipalNotUser {
                    principal_id: other_id,
                    upn: other_upn,
                    actual_kind: other_kind,
                },
            ) => (
                principal_id.as_ref(),
                upn.as_str(),
                actual_kind.graph_type(),
            )
                .cmp(&(
                    other_id.as_ref(),
                    other_upn.as_str(),
                    other_kind.graph_type(),
                )),
            (
                Self::UpnIdMismatch {
                    upn,
                    pinned_id,
                    observed_id,
                },
                Self::UpnIdMismatch {
                    upn: other_upn,
                    pinned_id: other_pinned,
                    observed_id: other_observed,
                },
            ) => (upn.as_str(), pinned_id.as_ref(), observed_id.as_ref()).cmp(&(
                other_upn.as_str(),
                other_pinned.as_ref(),
                other_observed.as_ref(),
            )),
            (
                Self::ConflictingUpnLookup { principal_id, upn },
                Self::ConflictingUpnLookup {
                    principal_id: other_id,
                    upn: other_upn,
                },
            ) => {
                (principal_id.as_ref(), upn.as_str()).cmp(&(other_id.as_ref(), other_upn.as_str()))
            }
            (
                Self::UpnNotFound { tenant_id, upn },
                Self::UpnNotFound {
                    tenant_id: other_tenant,
                    upn: other_upn,
                },
            ) => (tenant_id, upn.as_str()).cmp(&(other_tenant, other_upn.as_str())),
            _ => self.rank().cmp(&other.rank()),
        }
    }

    const fn rank(&self) -> u8 {
        match self {
            Self::LookupNotCompleted { .. } => 0,
            Self::DuplicateLookupRecords { .. } => 1,
            Self::UpnReturnedNonUser { .. } => 2,
            Self::UpnMismatch { .. } => 3,
            Self::PinnedPrincipalNotFound { .. } => 4,
            Self::PinnedIdMismatch { .. } => 5,
            Self::PinnedPrincipalNotUser { .. } => 6,
            Self::UpnIdMismatch { .. } => 7,
            Self::ConflictingUpnLookup { .. } => 8,
            Self::MissingIdentity => 9,
            Self::UpnNotFound { .. } => 10,
            Self::NilPrincipalId => 11,
        }
    }
}

impl Display for PrincipalResolutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LookupNotCompleted { tenant_id } => {
                write!(f, "No completed lookup is available in tenant {tenant_id}")
            }
            Self::DuplicateLookupRecords { tenant_id } => write!(
                f,
                "Ambiguous duplicate lookup records in tenant {tenant_id}"
            ),
            Self::UpnReturnedNonUser { upn, actual_kind } => write!(
                f,
                "UPN lookup for {upn} returned a non-user object ({actual_kind})"
            ),
            Self::UpnMismatch {
                upn,
                principal_id,
                observed,
            } => write!(
                f,
                "UPN lookup for {upn} returned an object without the same current UPN: principal {principal_id} has {:?}",
                observed.as_ref().map(UserPrincipalName::as_str)
            ),
            Self::PinnedPrincipalNotFound {
                tenant_id,
                principal_id,
            } => write!(
                f,
                "Pinned principal {principal_id} was not found in tenant {tenant_id}"
            ),
            Self::PinnedIdMismatch { expected, observed } => write!(
                f,
                "ID lookup for {expected} returned different principal {observed}"
            ),
            Self::PinnedPrincipalNotUser {
                principal_id,
                upn,
                actual_kind,
            } => write!(
                f,
                "Pinned principal {principal_id} is not a user and cannot be identified by UPN {upn} (observed {actual_kind})"
            ),
            Self::UpnIdMismatch {
                upn,
                pinned_id,
                observed_id,
            } => write!(
                f,
                "UPN {upn} resolves to {observed_id}, but the reference pins {pinned_id}"
            ),
            Self::ConflictingUpnLookup { principal_id, upn } => write!(
                f,
                "Conflicting fresh lookups: principal {principal_id} reports UPN {upn}, but that UPN was not found"
            ),
            Self::MissingIdentity => f.write_str("A principal requires id or userPrincipalName"),
            Self::UpnNotFound { tenant_id, upn } => write!(
                f,
                "User principal name {upn} was not found in tenant {tenant_id}"
            ),
            Self::NilPrincipalId => f.write_str("A directory lookup returned a nil principal ID"),
        }
    }
}

impl Error for PrincipalResolutionError {}
