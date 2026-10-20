use crate::domain::DesiredPrincipal;
use crate::domain::GroupRelationship;
use crate::planning::PrincipalResolutionError;
use cloud_terrastodon::azure_types::AzureTenantId;
use cloud_terrastodon::azure_types::EntraDirectoryObjectType;
use cloud_terrastodon::azure_types::EntraGroupId;
use cloud_terrastodon::azure_types::EntraPrincipalId;
use cloud_terrastodon::azure_types::UserPrincipalName;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

/// A structured reason why a group's proposed changes cannot be applied safely.
#[derive(Debug, Clone, Eq, PartialEq, facet::Facet)]
#[repr(C)]
#[facet(tag = "code", rename_all = "camelCase")]
pub enum GroupPlanBlocker {
    #[facet(rename_all = "camelCase")]
    MissingSnapshot {
        tenant_id: AzureTenantId,
        group_id: EntraGroupId,
    },
    #[facet(rename_all = "camelCase")]
    IncompleteExactInventory {
        relationship: GroupRelationship,
    },
    #[facet(rename_all = "camelCase")]
    GroupCannotBeOwner {
        principal_id: EntraPrincipalId,
    },
    #[facet(rename_all = "camelCase")]
    DuplicateDesiredPrincipal {
        relationship: GroupRelationship,
        principal_id: EntraPrincipalId,
    },
    #[facet(rename_all = "camelCase")]
    PrincipalResolutionFailed {
        relationship: GroupRelationship,
        reference: DesiredPrincipal,
        error: PrincipalResolutionError,
    },
    #[facet(rename_all = "camelCase")]
    DuplicateObservedPrincipal {
        relationship: GroupRelationship,
        principal_id: EntraPrincipalId,
    },
    #[facet(rename_all = "camelCase")]
    NilObservedPrincipal {
        relationship: GroupRelationship,
    },
    #[facet(rename_all = "camelCase")]
    ServicePrincipalPresenceUnknown {
        relationship: GroupRelationship,
        principal_id: EntraPrincipalId,
    },
    #[facet(rename_all = "camelCase")]
    UnsupportedPrincipalKind {
        relationship: GroupRelationship,
        principal_id: EntraPrincipalId,
        actual_kind: EntraDirectoryObjectType,
    },
    #[facet(rename_all = "camelCase")]
    UnsupportedGroupConfiguration {
        security_enabled: bool,
        mail_enabled: Option<bool>,
    },
    DynamicMembership,
    OnPremisesSynchronized,
    RoleAssignableGroup,
    UnknownRoleAssignability,
}

impl GroupPlanBlocker {
    /// Compare identity facts and annotations independently of rendered messages.
    pub(crate) fn compare(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                Self::MissingSnapshot {
                    tenant_id,
                    group_id,
                },
                Self::MissingSnapshot {
                    tenant_id: other_tenant,
                    group_id: other_group,
                },
            ) => (tenant_id, group_id.as_ref()).cmp(&(other_tenant, other_group.as_ref())),
            (
                Self::IncompleteExactInventory { relationship },
                Self::IncompleteExactInventory {
                    relationship: other_relationship,
                },
            )
            | (
                Self::NilObservedPrincipal { relationship },
                Self::NilObservedPrincipal {
                    relationship: other_relationship,
                },
            ) => relationship.as_str().cmp(other_relationship.as_str()),
            (
                Self::GroupCannotBeOwner { principal_id },
                Self::GroupCannotBeOwner {
                    principal_id: other_id,
                },
            ) => principal_id.as_ref().cmp(other_id.as_ref()),
            (
                Self::DuplicateDesiredPrincipal {
                    relationship,
                    principal_id,
                },
                Self::DuplicateDesiredPrincipal {
                    relationship: other_relationship,
                    principal_id: other_id,
                },
            )
            | (
                Self::DuplicateObservedPrincipal {
                    relationship,
                    principal_id,
                },
                Self::DuplicateObservedPrincipal {
                    relationship: other_relationship,
                    principal_id: other_id,
                },
            )
            | (
                Self::ServicePrincipalPresenceUnknown {
                    relationship,
                    principal_id,
                },
                Self::ServicePrincipalPresenceUnknown {
                    relationship: other_relationship,
                    principal_id: other_id,
                },
            ) => (relationship.as_str(), principal_id.as_ref())
                .cmp(&(other_relationship.as_str(), other_id.as_ref())),
            (
                Self::PrincipalResolutionFailed {
                    relationship,
                    reference,
                    error,
                },
                Self::PrincipalResolutionFailed {
                    relationship: other_relationship,
                    reference: other_reference,
                    error: other_error,
                },
            ) => (
                relationship.as_str(),
                reference.id.as_ref().map(EntraPrincipalId::as_ref),
                reference
                    .user_principal_name
                    .as_ref()
                    .map(UserPrincipalName::as_str),
                reference.display_name.as_deref(),
            )
                .cmp(&(
                    other_relationship.as_str(),
                    other_reference.id.as_ref().map(EntraPrincipalId::as_ref),
                    other_reference
                        .user_principal_name
                        .as_ref()
                        .map(UserPrincipalName::as_str),
                    other_reference.display_name.as_deref(),
                ))
                .then_with(|| error.compare(other_error)),
            (
                Self::UnsupportedGroupConfiguration {
                    security_enabled,
                    mail_enabled,
                },
                Self::UnsupportedGroupConfiguration {
                    security_enabled: other_security,
                    mail_enabled: other_mail,
                },
            ) => (security_enabled, mail_enabled).cmp(&(other_security, other_mail)),
            (
                Self::UnsupportedPrincipalKind {
                    relationship,
                    principal_id,
                    actual_kind,
                },
                Self::UnsupportedPrincipalKind {
                    relationship: other_relationship,
                    principal_id: other_id,
                    actual_kind: other_kind,
                },
            ) => (
                relationship.as_str(),
                principal_id.as_ref(),
                actual_kind.graph_type(),
            )
                .cmp(&(
                    other_relationship.as_str(),
                    other_id.as_ref(),
                    other_kind.graph_type(),
                )),
            _ => self.rank().cmp(&other.rank()),
        }
    }

    const fn rank(&self) -> u8 {
        match self {
            Self::MissingSnapshot { .. } => 0,
            Self::IncompleteExactInventory { .. } => 1,
            Self::GroupCannotBeOwner { .. } => 2,
            Self::DuplicateDesiredPrincipal { .. } => 3,
            Self::PrincipalResolutionFailed { .. } => 4,
            Self::DuplicateObservedPrincipal { .. } => 5,
            Self::NilObservedPrincipal { .. } => 6,
            Self::ServicePrincipalPresenceUnknown { .. } => 7,
            Self::UnsupportedGroupConfiguration { .. } => 8,
            Self::DynamicMembership => 9,
            Self::OnPremisesSynchronized => 10,
            Self::RoleAssignableGroup => 11,
            Self::UnknownRoleAssignability => 12,
            Self::UnsupportedPrincipalKind { .. } => 13,
        }
    }
}

impl Display for GroupPlanBlocker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSnapshot { tenant_id, group_id } => write!(
                f,
                "No fresh snapshot is available for group {group_id} in tenant {tenant_id}"
            ),
            Self::IncompleteExactInventory { relationship } => write!(
                f,
                "Exact {relationship} reconciliation requires a complete inventory; the selected Graph inventory can omit objects"
            ),
            Self::GroupCannotBeOwner { principal_id } => write!(
                f,
                "Group {principal_id} cannot be assigned as an owner; owners must be users or service principals"
            ),
            Self::DuplicateDesiredPrincipal { relationship, principal_id } => write!(
                f,
                "Multiple {relationship} references resolve to the same principal {principal_id}"
            ),
            Self::PrincipalResolutionFailed { relationship, error, .. } => write!(
                f,
                "Cannot resolve {relationship}: {error}"
            ),
            Self::DuplicateObservedPrincipal { relationship, principal_id } => write!(
                f,
                "The {relationship} snapshot contains duplicate principal {principal_id}"
            ),
            Self::NilObservedPrincipal { relationship } => write!(
                f,
                "The {relationship} snapshot contains a nil principal ID"
            ),
            Self::ServicePrincipalPresenceUnknown { relationship, principal_id } => write!(
                f,
                "Cannot establish whether service principal {principal_id} is already among {relationship}; the selected Graph inventory can omit service principals"
            ),
            Self::UnsupportedPrincipalKind { relationship, principal_id, actual_kind } => write!(
                f,
                "Reconciliation of {actual_kind} principal {principal_id} among {relationship} is outside this tool's supported scope"
            ),
            Self::UnsupportedGroupConfiguration { security_enabled, mail_enabled } => write!(
                f,
                "Only non-mail-enabled security groups are supported; observed securityEnabled={security_enabled}, mailEnabled={mail_enabled:?}"
            ),
            Self::DynamicMembership => f.write_str(
                "Dynamic-membership groups cannot be reconciled through explicit membership changes"
            ),
            Self::OnPremisesSynchronized => f.write_str(
                "On-premises-synchronized groups must be managed at their authoritative source"
            ),
            Self::RoleAssignableGroup => f.write_str(
                "Role-assignable groups are outside this tool's supported reconciliation scope"
            ),
            Self::UnknownRoleAssignability => f.write_str(
                "The group's role assignability is unknown; eligibility cannot be confirmed"
            ),
        }
    }
}

impl Error for GroupPlanBlocker {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PrincipalResolutionFailed { error, .. } => Some(error),
            _ => None,
        }
    }
}
