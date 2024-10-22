use crate::prelude::Fake;
use crate::prelude::ManagementGroupScoped;
use crate::prelude::ResourceGroupScoped;
use crate::prelude::ResourceScoped;
use crate::prelude::RoleDefinitionId;
use crate::prelude::SubscriptionScoped;
use crate::prelude::Unscoped;
use crate::scopes::try_from_expanded_hierarchy_scoped;
use crate::scopes::HasPrefix;
use crate::scopes::HasScope;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromManagementGroupScoped;
use crate::scopes::TryFromResourceGroupScoped;
use crate::scopes::TryFromResourceScoped;
use crate::scopes::TryFromSubscriptionScoped;
use crate::scopes::TryFromUnscoped;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_core_tofu_types::prelude::Sanitizable;
use cloud_terrastodon_core_tofu_types::prelude::TofuAzureRMResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu_types::prelude::TofuResourceReference;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_json::Value;
use uuid::Uuid;

pub const ROLE_ASSIGNMENT_ID_PREFIX: &str = "/providers/Microsoft.Authorization/roleAssignments/";

// TODO: learn how to make a derive or other macro to simplify this, cause this isn't going to scale well lol

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UnscopedRoleAssignmentId {
    expanded: String,
}
impl UnscopedRoleAssignmentId {
    pub fn new(uuid: &Uuid) -> Self {
        UnscopedRoleAssignmentId {
            expanded: format!("{ROLE_ASSIGNMENT_ID_PREFIX}{}", uuid.as_hyphenated()),
        }
    }
}
impl Unscoped for UnscopedRoleAssignmentId {}
impl NameValidatable for UnscopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for UnscopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl TryFromUnscoped for UnscopedRoleAssignmentId {
    unsafe fn new_unscoped_unchecked(expanded: &str) -> Self {
        UnscopedRoleAssignmentId {
            expanded: expanded.to_string(),
        }
    }
}
impl Fake for UnscopedRoleAssignmentId {
    fn fake() -> Self {
        UnscopedRoleAssignmentId::new(&Uuid::nil())
    }
}
impl Scope for UnscopedRoleAssignmentId {
    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        UnscopedRoleAssignmentId::try_from_expanded_unscoped(expanded)
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::Unscoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ManagementGroupScopedRoleAssignmentId {
    expanded: String,
}
impl NameValidatable for ManagementGroupScopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for ManagementGroupScopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl TryFromManagementGroupScoped for ManagementGroupScopedRoleAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self {
        ManagementGroupScopedRoleAssignmentId {
            expanded: expanded.to_owned(),
        }
    }
}
impl Scope for ManagementGroupScopedRoleAssignmentId {
    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ManagementGroupScopedRoleAssignmentId::try_from_expanded_management_group_scoped(expanded)
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::ManagementGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}
impl ManagementGroupScoped for ManagementGroupScopedRoleAssignmentId {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubscriptionScopedRoleAssignmentId {
    expanded: String,
}
impl NameValidatable for SubscriptionScopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for SubscriptionScopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl TryFromSubscriptionScoped for SubscriptionScopedRoleAssignmentId {
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self {
        SubscriptionScopedRoleAssignmentId {
            expanded: expanded.to_owned(),
        }
    }
}
impl Scope for SubscriptionScopedRoleAssignmentId {
    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        SubscriptionScopedRoleAssignmentId::try_from_expanded_subscription_scoped(expanded)
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::SubscriptionScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}
impl SubscriptionScoped for SubscriptionScopedRoleAssignmentId {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupScopedRoleAssignmentId {
    expanded: String,
    // subscription_id: &'a str,
    // resource_group_name: &'a str,
}

impl NameValidatable for ResourceGroupScopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for ResourceGroupScopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl TryFromResourceGroupScoped for ResourceGroupScopedRoleAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self {
        ResourceGroupScopedRoleAssignmentId {
            expanded: expanded.to_owned(),
        }
    }
}
impl Scope for ResourceGroupScopedRoleAssignmentId {
    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceGroupScopedRoleAssignmentId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::ResourceGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}
impl SubscriptionScoped for ResourceGroupScopedRoleAssignmentId {}
impl ResourceGroupScoped for ResourceGroupScopedRoleAssignmentId {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceScopedRoleAssignmentId {
    expanded: String,
    // resource_id: &'a str
}

impl NameValidatable for ResourceScopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for ResourceScopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl TryFromResourceScoped for ResourceScopedRoleAssignmentId {
    unsafe fn new_resource_scoped_unchecked(expanded: &str) -> Self {
        ResourceScopedRoleAssignmentId {
            expanded: expanded.to_owned(),
        }
    }
}
impl Scope for ResourceScopedRoleAssignmentId {
    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceScopedRoleAssignmentId::try_from_expanded_resource_scoped(expanded)
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::ResourceScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}
impl SubscriptionScoped for ResourceScopedRoleAssignmentId {}
impl ResourceGroupScoped for ResourceScopedRoleAssignmentId {}
impl ResourceScoped for ResourceScopedRoleAssignmentId {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RoleAssignmentId {
    Unscoped(UnscopedRoleAssignmentId),
    ManagementGroupScoped(ManagementGroupScopedRoleAssignmentId),
    SubscriptionScoped(SubscriptionScopedRoleAssignmentId),
    ResourceGroupScoped(ResourceGroupScopedRoleAssignmentId),
    ResourceScoped(ResourceScopedRoleAssignmentId),
}

impl NameValidatable for RoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for RoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl Fake for RoleAssignmentId {
    fn fake() -> Self {
        RoleAssignmentId::Unscoped(UnscopedRoleAssignmentId::fake())
    }
}

impl TryFromUnscoped for RoleAssignmentId {
    unsafe fn new_unscoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::Unscoped(UnscopedRoleAssignmentId {
            expanded: expanded.to_string(),
        })
    }
}
impl TryFromResourceGroupScoped for RoleAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::ResourceGroupScoped(ResourceGroupScopedRoleAssignmentId {
            expanded: expanded.to_string(),
        })
    }
}
impl TryFromResourceScoped for RoleAssignmentId {
    unsafe fn new_resource_scoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::ResourceScoped(ResourceScopedRoleAssignmentId {
            expanded: expanded.to_string(),
        })
    }
}
impl TryFromSubscriptionScoped for RoleAssignmentId {
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::SubscriptionScoped(SubscriptionScopedRoleAssignmentId {
            expanded: expanded.to_string(),
        })
    }
}
impl TryFromManagementGroupScoped for RoleAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::ManagementGroupScoped(ManagementGroupScopedRoleAssignmentId {
            expanded: expanded.to_string(),
        })
    }
}

impl Scope for RoleAssignmentId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        try_from_expanded_hierarchy_scoped(expanded)
    }

    fn expanded_form(&self) -> &str {
        match self {
            Self::Unscoped(UnscopedRoleAssignmentId { expanded }) => expanded,
            Self::ResourceGroupScoped(ResourceGroupScopedRoleAssignmentId { expanded }) => expanded,
            Self::SubscriptionScoped(SubscriptionScopedRoleAssignmentId { expanded }) => expanded,
            Self::ManagementGroupScoped(ManagementGroupScopedRoleAssignmentId { expanded }) => {
                expanded
            }
            Self::ResourceScoped(ResourceScopedRoleAssignmentId { expanded }) => expanded,
        }
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::RoleAssignment(self.clone())
    }
}

impl Serialize for RoleAssignmentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for RoleAssignmentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = RoleAssignmentId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
        Ok(id)
    }
}

fn stupid_uuid_deserialize<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<&str> = Option::deserialize(deserializer)?;
    if let Some(s) = s {
        if s.is_empty() {
            Ok(None)
        } else {
            Uuid::parse_str(s)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
    } else {
        Ok(None)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ThinRoleAssignment {
    pub id: RoleAssignmentId,
    pub scope: String,
    pub role_definition_id: RoleDefinitionId,
    pub principal_id: Uuid,
}

impl Fake for ThinRoleAssignment {
    fn fake() -> Self {
        ThinRoleAssignment {
            id: RoleAssignmentId::fake(),
            scope: "".to_owned(), 
            role_definition_id: RoleDefinitionId::fake(),
            principal_id: Uuid::nil(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleAssignment {
    pub condition: Option<Value>,
    #[serde(rename = "conditionVersion")]
    pub condition_version: Option<Value>,
    #[serde(rename = "createdBy", deserialize_with = "stupid_uuid_deserialize")]
    pub created_by: Option<Uuid>,
    #[serde(rename = "createdOn")]
    pub created_on: DateTime<Utc>,
    #[serde(rename = "delegatedManagedIdentityResourceId")]
    pub delegated_managed_identity_resource_id: Option<Value>,
    pub description: Option<Value>,
    pub id: RoleAssignmentId,
    pub name: Uuid,
    #[serde(rename = "principalId")]
    pub principal_id: Uuid,
    #[serde(rename = "principalName")]
    pub principal_name: String,
    #[serde(rename = "principalType")]
    pub principal_type: String,
    #[serde(rename = "roleDefinitionId")]
    pub role_definition_id: String,
    #[serde(rename = "roleDefinitionName")]
    pub role_definition_name: String,
    pub scope: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "updatedBy", deserialize_with = "stupid_uuid_deserialize")]
    pub updated_by: Option<Uuid>,
    #[serde(rename = "updatedOn")]
    pub updated_on: DateTime<Utc>,
}

impl HasScope for RoleAssignment {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &RoleAssignment {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for ThinRoleAssignment {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &ThinRoleAssignment {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for RoleAssignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.role_definition_name)?;
        f.write_str(" for ")?;
        f.write_str(&self.principal_name)?;
        f.write_str(" (")?;
        f.write_str(self.principal_id.to_string().as_str())?;
        f.write_str(")")?;
        Ok(())
    }
}

impl From<RoleAssignment> for TofuImportBlock {
    fn from(role_assignment: RoleAssignment) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            // Tofu doesn't like the case variation, https://github.com/hashicorp/terraform-provider-azurerm/issues/26907
            id: role_assignment
                .id
                .expanded_form()
                .replace("/RoleAssignments/", "/roleAssignments/"),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::RoleAssignment,
                name: format!(
                    "{}__{}",
                    role_assignment.name,
                    role_assignment.id.expanded_form()
                )
                .sanitize(),
            },
        }
    }
}
impl From<ThinRoleAssignment> for TofuImportBlock {
    fn from(role_assignment: ThinRoleAssignment) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            // Tofu doesn't like the case variation, https://github.com/hashicorp/terraform-provider-azurerm/issues/26907
            id: role_assignment
                .id
                .expanded_form()
                .replace("/RoleAssignments/", "/roleAssignments/"),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::RoleAssignment,
                name: role_assignment.id.short_form().sanitize(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = RoleAssignmentId::Unscoped(UnscopedRoleAssignmentId {
            expanded: format!(
                "{}{}",
                ROLE_ASSIGNMENT_ID_PREFIX, "55555555-5555-5555-5555-555555555555"
            ),
        });
        let id: RoleAssignmentId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }
    #[test]
    fn deserializes2() -> Result<()> {
        let nil = Uuid::nil();
        let expanded = RoleAssignmentId::ResourceScoped(ResourceScopedRoleAssignmentId {
            expanded: format!(
                "/subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Network/virtualNetworks/MY-VNET/subnets/MY-Subnet/providers/Microsoft.Authorization/roleAssignments/{nil}",
            ),
        });
        let id: RoleAssignmentId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }
}
