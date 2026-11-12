use crate::AzureAuditPasswordCredentialStatus;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure::EntraPrincipalId;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::RoleAssignmentId;
use cloud_terrastodon_azure::ScopeImpl;
use facet::Facet;

/// A policy finding with the identities and observations needed to investigate it.
#[derive(Debug, Arbitrary, Facet)]
#[repr(u8)]
pub enum AzureAuditFinding {
    UnknownRoleAssignmentPrincipal {
        principal_id: EntraPrincipalId,
        role_definition_name: String,
        role_assignment_id: RoleAssignmentId,
        scope: ScopeImpl,
    },
    ServicePrincipalPassword {
        principal_id: EntraServicePrincipalObjectId,
        principal_name: String,
        password_cred_end_date: DateTime<Utc>,
        days_until_expiry: i64,
        status: AzureAuditPasswordCredentialStatus,
    },
    ResourceTagMismatch {
        resource: ScopeImpl,
        parent: ScopeImpl,
        tag_key: String,
        resource_tag_value: String,
        parent_tag_value: String,
    },
    AdminWithoutCorrespondingAccount {
        principal_id: EntraUserId,
        principal_name: String,
        principal_other_mails: Vec<String>,
    },
}

impl AzureAuditFinding {
    pub fn message(&self) -> &'static str {
        match self {
            Self::UnknownRoleAssignmentPrincipal { .. } => {
                "Found role assignment with unknown principal"
            }
            Self::ServicePrincipalPassword { status, .. } => match status {
                AzureAuditPasswordCredentialStatus::ExpiredWithoutReplacement => {
                    "Service principal has an expired password credential and no valid credentials"
                }
                AzureAuditPasswordCredentialStatus::ExpiringWithoutReplacement => {
                    "Service principal has a soon-to-expire password credential and no valid credentials"
                }
                AzureAuditPasswordCredentialStatus::ExpiredWithReplacement => {
                    "Service principal has an expired password credential but also has valid credentials; consider cleaning up old credentials"
                }
            },
            Self::ResourceTagMismatch { .. } => {
                "Resource tag value does not match parent resource tag value"
            }
            Self::AdminWithoutCorrespondingAccount { .. } => {
                "Admin user account does not have a corresponding non-admin account according to other_mails property"
            }
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureAuditFinding);
cloud_terrastodon_registry::register_arbitrary!(AzureAuditFinding);
