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
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_json::Value;
use tofu_types::prelude::Sanitizable;
use tofu_types::prelude::TofuAzureRMResourceKind;
use tofu_types::prelude::TofuImportBlock;
use tofu_types::prelude::TofuProviderReference;
use tofu_types::prelude::TofuResourceReference;
use uuid::Uuid;

pub const ROLE_ASSIGNMENT_ID_PREFIX: &str = "/providers/Microsoft.Authorization/roleAssignments/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RoleAssignmentId {
    Unscoped { expanded: String },
    ManagementGroupScoped { expanded: String },
    SubscriptionScoped { expanded: String },
    ResourceGroupScoped { expanded: String },
    ResourceScoped { expanded: String },
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
impl TryFromUnscoped for RoleAssignmentId {
    unsafe fn new_unscoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::Unscoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromResourceGroupScoped for RoleAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::ResourceGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromResourceScoped for RoleAssignmentId {
    unsafe fn new_resource_scoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::ResourceScoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromSubscriptionScoped for RoleAssignmentId {
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::SubscriptionScoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromManagementGroupScoped for RoleAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::ManagementGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl Scope for RoleAssignmentId {
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

    fn short_form(&self) -> &str {
        self.expanded_form()
            .rsplit_once('/')
            .expect("no slash found, structure should have been validated at construction")
            .1
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
    fn from(resource_group: RoleAssignment) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: resource_group.id.expanded_form().to_owned(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::RoleAssignment,
                name: format!(
                    "{}__{}",
                    resource_group.name,
                    resource_group.id.expanded_form()
                )
                .sanitize(),
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
        let expanded = RoleAssignmentId::Unscoped {
            expanded: format!(
                "{}{}",
                ROLE_ASSIGNMENT_ID_PREFIX, "55555555-5555-5555-5555-555555555555"
            ),
        };
        let id: RoleAssignmentId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }
    #[test]
    fn deserializes2() -> Result<()> {
        let nil = Uuid::nil();
        let expanded = RoleAssignmentId::ResourceScoped {
            expanded: format!(
                "/subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Network/virtualNetworks/MY-VNET/subnets/MY-Subnet/providers/Microsoft.Authorization/roleAssignments/{nil}",
            ),
        };
        let id: RoleAssignmentId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }
}
