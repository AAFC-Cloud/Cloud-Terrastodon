use crate::prelude::RolePermissionAction;
use crate::prelude::UnifiedRoleDefinitionId;
use crate::serde_helpers::deserialize_default_if_null;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use tracing::warn;

/// An Entra role definition.
/// 
/// Not to be confused with an Azure RBAC role definition.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedRoleDefinition {
    // pub allowed_principal_types: Value, // is present in $metadata but the API isn't giving it :P
    pub description: String,
    pub display_name: String,
    pub is_built_in: bool,
    pub is_enabled: bool,
    pub is_privileged: bool,
    pub resource_scopes: Vec<String>,
    pub role_permissions: Vec<UnifiedRolePermission>,
    pub template_id: UnifiedRoleDefinitionId,
    pub version: Option<String>,
    pub inherits_permissions_from: Vec<UnifiedRoleDefinitionIdReference>,
    #[serde(skip)]
    pub canonicalized: bool,
}
impl UnifiedRoleDefinition {
    pub fn canonicalize(
        &mut self,
        role_definitions: &HashMap<UnifiedRoleDefinitionId, UnifiedRoleDefinition>,
    ) {
        let mut remaining_parents = std::mem::take(&mut self.inherits_permissions_from);
        while let Some(parent_id) = remaining_parents.pop() {
            let parent = match role_definitions.get(&parent_id.id) {
                Some(r) => r.clone(),
                None => {
                    warn!(
                        "Role definition {} not found in our collection of {} roles",
                        parent_id.id,
                        role_definitions.len()
                    );
                    continue;
                }
            };
            self.inherits_permissions_from.push(parent_id);
            self.role_permissions.extend(parent.role_permissions);
            remaining_parents.extend(parent.inherits_permissions_from);
        }
    }
}

impl UnifiedRoleDefinition {
    pub fn satisfies(&self, actions: &[RolePermissionAction]) -> bool {
        if !self.canonicalized && !self.inherits_permissions_from.is_empty() {
            warn!(
                "Checking if non-canonicalized role definition {} ({}) satisfies actions, this role definition has {} parents",
                self.template_id,
                self.display_name,
                self.inherits_permissions_from.len()
            );
        }
        // Correctness pitfall: we don't test resource_scopes here.
        // I've found no role definition that has resource scopes other than ["/"]
        for permission in &self.role_permissions {
            if permission.satisfies(actions) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnifiedRoleDefinitionIdReference {
    pub id: UnifiedRoleDefinitionId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedRolePermission {
    pub condition: Option<String>,
    pub allowed_resource_actions: Vec<RolePermissionAction>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_default_if_null")]
    pub excluded_resource_actions: Vec<RolePermissionAction>,
}

impl UnifiedRolePermission {
    pub fn satisfies(&self, actions: &[RolePermissionAction]) -> bool {
        for not_action in &self.excluded_resource_actions {
            for action in actions {
                if not_action.satisfies(action) {
                    return false;
                }
            }
        }
        for action in actions {
            let mut satisfied = false;
            for self_action in &self.allowed_resource_actions {
                if self_action.satisfies(action) {
                    satisfied = true;
                    break;
                }
            }
            if !satisfied {
                return false;
            }
        }
        true
    }
}
