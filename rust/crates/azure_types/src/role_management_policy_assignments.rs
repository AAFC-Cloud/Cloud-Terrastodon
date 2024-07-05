use crate::prelude::RoleDefinitionId;
use crate::prelude::RoleDefinitionKind;
use crate::prelude::RoleManagementPolicyId;
use crate::scopes::try_from_expanded_hierarchy_scoped;
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
use uuid::Uuid;

pub const ROLE_MANAGEMENT_POLICY_ASSIGNMENT_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/roleManagementPolicyAssignments/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RoleManagementPolicyAssignmentId {
    Unscoped { expanded: String },
    ManagementGroupScoped { expanded: String },
    SubscriptionScoped { expanded: String },
    ResourceGroupScoped { expanded: String },
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
        }
    }

    fn short_form(&self) -> &str {
        self.expanded_form()
            .rsplit_once('/')
            .expect("no slash found, structure should have been validated at construction")
            .1
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
pub struct RoleManagementPolicyAssignmentProperties {
    pub scope: String,
    #[serde(rename = "roleDefinitionId")]
    pub role_definition_id: String,
    #[serde(rename = "policyId")]
    pub policy_id: String,
    #[serde(rename = "effectiveRules")]
    pub effective_rules: Vec<Value>,
    #[serde(rename = "policyAssignmentProperties")]
    pub policy_assignment_properties: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoleManagementPolicyAssignment {
    pub properties: RoleManagementPolicyAssignmentProperties,
    pub name: String,
    pub id: RoleManagementPolicyAssignmentId,
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
