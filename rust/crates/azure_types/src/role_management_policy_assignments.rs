use crate::prelude::try_from_expanded_hierarchy_scoped;
use crate::prelude::RoleDefinitionId;
use crate::prelude::RoleDefinitionKind;
use crate::prelude::RoleManagementPolicyId;
use crate::prelude::TryFromResourceScoped;
use crate::scopes::HasPrefix;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromManagementGroupScoped;
use crate::scopes::TryFromResourceGroupScoped;
use crate::scopes::TryFromSubscriptionScoped;
use crate::scopes::TryFromUnscoped;
use anyhow::anyhow;
use anyhow::Result;
use core::any::type_name;
use serde::de::Error;
use serde::de::Visitor;
use serde::de::{self};
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use uuid::Uuid;

pub const ROLE_MANAGEMENT_POLICY_ASSIGNMENT_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/roleManagementPolicyAssignments/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RoleManagementPolicyAssignmentId {
    Unscoped { expanded: String },
    ManagementGroupScoped { expanded: String },
    SubscriptionScoped { expanded: String },
    ResourceGroupScoped { expanded: String },
    ResourceScoped { expanded: String },
}
impl NameValidatable for RoleManagementPolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        let (role_management_policy, role_management_policy_assignment) =
            name.split_once('_').ok_or_else(|| {
                anyhow!(
                    "{} names are two uuids joined by an underscore",
                    type_name::<RoleManagementPolicyAssignmentId>()
                )
            })?;
        Uuid::parse_str(role_management_policy)?;
        Uuid::parse_str(role_management_policy_assignment)?;
        Ok(())
    }
}
impl HasPrefix for RoleManagementPolicyAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ASSIGNMENT_ID_PREFIX
    }
}
impl TryFromUnscoped for RoleManagementPolicyAssignmentId {
    unsafe fn new_unscoped_unchecked(expanded: &str) -> Self {
        RoleManagementPolicyAssignmentId::Unscoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromResourceGroupScoped for RoleManagementPolicyAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self {
        RoleManagementPolicyAssignmentId::ResourceGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl TryFromResourceScoped for RoleManagementPolicyAssignmentId {
    unsafe fn new_resource_scoped_unchecked(expanded: &str) -> Self {
        RoleManagementPolicyAssignmentId::ResourceScoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromSubscriptionScoped for RoleManagementPolicyAssignmentId {
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self {
        RoleManagementPolicyAssignmentId::SubscriptionScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl TryFromManagementGroupScoped for RoleManagementPolicyAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self {
        RoleManagementPolicyAssignmentId::ManagementGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl Scope for RoleManagementPolicyAssignmentId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        try_from_expanded_hierarchy_scoped(expanded)
    }

    fn expanded_form(&self) -> &str {
        match self {
            Self::Unscoped { expanded } => expanded,
            Self::ResourceGroupScoped { expanded } => expanded,
            Self::SubscriptionScoped { expanded } => expanded,
            Self::ManagementGroupScoped { expanded } => expanded,
            Self::ResourceScoped { expanded } => expanded,
        }
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleManagementPolicyAssignment
    }
    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::RoleManagementPolicyAssignment(self.clone())
    }
}

impl Serialize for RoleManagementPolicyAssignmentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for RoleManagementPolicyAssignmentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = RoleManagementPolicyAssignmentId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
        Ok(id)
    }
}

#[derive(Debug)]
pub enum RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind {
    ManagementGroup,
    Subscription,
    ResourceGroup,
    Other(String),
}

impl Serialize for RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::ManagementGroup => serializer.serialize_str("managementgroup"),
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::Subscription => serializer.serialize_str("subscription"),
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::ResourceGroup => serializer.serialize_str("resourcegroup"),
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::Other(ref s) => serializer.serialize_str(s),
        }
    }
}

struct RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKindVisitor;

impl<'de> Visitor<'de>
    for RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKindVisitor
{
    type Value = RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing the resource kind")
    }

    fn visit_str<E>(
        self,
        value: &str,
    ) -> Result<RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind, E>
    where
        E: de::Error,
    {
        Ok(match value {
            "managementgroup" => RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::ManagementGroup,
            "subscription" => RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::Subscription,
            "resourcegroup" => RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::ResourceGroup,
            other => RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::Other(other.to_string()),
        })
    }
}

impl<'de> Deserialize<'de>
    for RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKindVisitor,
        )
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScope {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesRoleDefinition {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: RoleDefinitionId,
    #[serde(rename = "type")]
    pub kind: RoleDefinitionKind,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesPolicy {
    pub id: RoleManagementPolicyId,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleManagementPolicyAssignmentPropertiesPolicyAssignmentProperties {
    pub scope: RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScope,
    #[serde(rename = "roleDefinition")]
    pub role_definition:
        RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesRoleDefinition,
    pub policy: RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesPolicy,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoleManagementPolicyAssignmentPropertiesEffectiveRuleId {
    #[serde(rename = "Enablement_Admin_Eligibility")]
    EnablementAdminEligibility,
    #[serde(rename = "Expiration_Admin_Eligibility")]
    ExpirationAdminEligibility,
    #[serde(rename = "Notification_Admin_Admin_Eligibility")]
    NotificationAdminAdminEligibility,
    #[serde(rename = "Notification_Requestor_Admin_Eligibility")]
    NotificationRequestorAdminEligibility,
    #[serde(rename = "Notification_Approver_Admin_Eligibility")]
    NotificationApproverAdminEligibility,
    #[serde(rename = "Enablement_Admin_Assignment")]
    EnablementAdminAssignment,
    #[serde(rename = "Expiration_Admin_Assignment")]
    ExpirationAdminAssignment,
    #[serde(rename = "Notification_Admin_Admin_Assignment")]
    NotificationAdminAdminAssignment,
    #[serde(rename = "Notification_Requestor_Admin_Assignment")]
    NotificationRequestorAdminAssignment,
    #[serde(rename = "Notification_Approver_Admin_Assignment")]
    NotificationApproverAdminAssignment,
    #[serde(rename = "Approval_EndUser_Assignment")]
    ApprovalEnduserAssignment,
    #[serde(rename = "AuthenticationContext_EndUser_Assignment")]
    AuthenticationcontextEnduserAssignment,
    #[serde(rename = "Enablement_EndUser_Assignment")]
    EnablementEnduserAssignment,
    #[serde(rename = "Expiration_EndUser_Assignment")]
    ExpirationEnduserAssignment,
    #[serde(rename = "Notification_Admin_EndUser_Assignment")]
    NotificationAdminEnduserAssignment,
    #[serde(rename = "Notification_Requestor_EndUser_Assignment")]
    NotificationRequestorEnduserAssignment,
    #[serde(rename = "Notification_Approver_EndUser_Assignment")]
    NotificationApproverEnduserAssignment,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "ruleType")]
pub enum RoleManagementPolicyAssignmentPropertiesEffectiveRule {
    RoleManagementPolicyEnablementRule,
    RoleManagementPolicyExpirationRule {
        id: RoleManagementPolicyAssignmentPropertiesEffectiveRuleId,
        #[serde(rename = "isExpirationRequired")]
        is_expiration_required: bool,
        #[serde(rename = "maximumDuration")]
        maximum_duration: iso8601_duration::Duration,
        target: Value,
    },
    RoleManagementPolicyNotificationRule,
    RoleManagementPolicyApprovalRule,
    RoleManagementPolicyAuthenticationContextRule,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleManagementPolicyAssignmentProperties {
    pub scope: String,
    #[serde(rename = "roleDefinitionId")]
    pub role_definition_id: String,
    #[serde(rename = "policyId")]
    pub policy_id: String,
    #[serde(rename = "effectiveRules")]
    pub effective_rules: Vec<RoleManagementPolicyAssignmentPropertiesEffectiveRule>,
    #[serde(rename = "policyAssignmentProperties")]
    pub policy_assignment_properties: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleManagementPolicyAssignment {
    pub properties: RoleManagementPolicyAssignmentProperties,
    pub name: String,
    pub id: RoleManagementPolicyAssignmentId,
}

impl RoleManagementPolicyAssignment {
    pub fn get_maximum_activation_duration(&self) -> Option<Duration> {
        for rule in &self.properties.effective_rules {
            if let RoleManagementPolicyAssignmentPropertiesEffectiveRule::RoleManagementPolicyExpirationRule {
                 id: RoleManagementPolicyAssignmentPropertiesEffectiveRuleId::ExpirationEnduserAssignment, maximum_duration, ..
            } = rule {
                return maximum_duration.to_std();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() -> Result<()> {
        let id = format!("/providers/Microsoft.Management/managementGroups/{}/providers/Microsoft.Authorization/roleManagementPolicyAssignments/{}_{}",
            Uuid::nil(),
            Uuid::nil(),
            Uuid::nil(),
        );
        RoleManagementPolicyAssignmentId::try_from_expanded(&id)?;
        Ok(())
    }
}
