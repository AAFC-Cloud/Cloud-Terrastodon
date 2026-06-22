use crate::RolePermissionAction;
use crate::UnifiedRoleDefinitionId;
use std::collections::HashMap;
use tracing::warn;

/// An Entra role definition.
///
/// Not to be confused with an Azure RBAC role definition.
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
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
    #[facet(skip)]
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

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
pub struct UnifiedRoleDefinitionIdReference {
    pub id: UnifiedRoleDefinitionId,
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct UnifiedRolePermission {
    pub condition: Option<String>,
    pub allowed_resource_actions: Vec<RolePermissionAction>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<RolePermissionAction>)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_definition_json_round_trips_through_facet() -> eyre::Result<()> {
        let json = r#"
        {
            "description": "Can read application registrations",
            "displayName": "Application Reader",
            "isBuiltIn": true,
            "isEnabled": true,
            "isPrivileged": false,
            "resourceScopes": ["/"],
            "rolePermissions": [
                {
                    "condition": null,
                    "allowedResourceActions": [
                        "microsoft.directory/applications/basic/read"
                    ],
                    "excludedResourceActions": null
                }
            ],
            "templateId": "00000000-0000-0000-0000-000000000001",
            "version": "1",
            "inheritsPermissionsFrom": [
                {
                    "id": "00000000-0000-0000-0000-000000000002"
                }
            ]
        }
        "#;

        let role_definition = facet_json::from_str::<UnifiedRoleDefinition>(json)?;
        assert!(!role_definition.canonicalized);
        assert_eq!(
            role_definition.role_permissions[0].excluded_resource_actions,
            Vec::<RolePermissionAction>::new()
        );

        let serialized = facet_json::to_string(&role_definition)?;
        assert!(!serialized.contains("canonicalized"));
        let reparsed = facet_json::from_str::<UnifiedRoleDefinition>(&serialized)?;
        assert_eq!(role_definition, reparsed);
        Ok(())
    }
}
