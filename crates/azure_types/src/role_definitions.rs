use crate::prelude::RoleDefinitionId;
use crate::prelude::RolePermissionAction;
use crate::prelude::RolePermissions;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use cloud_terrastodon_hcl_types::prelude::AzureRmResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HclImportBlock;
use cloud_terrastodon_hcl_types::prelude::HclProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum RoleDefinitionKind {
    BuiltInRole,
    CustomRole,
}

/// An Azure RBAC role definition.
///
/// Not to be confused with an Entra role definition.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RoleDefinition {
    pub id: RoleDefinitionId,
    pub display_name: String,
    pub description: String,
    pub assignable_scopes: Vec<String>,
    pub permissions: Vec<RolePermissions>,
    pub kind: RoleDefinitionKind,
}

impl RoleDefinition {
    pub fn satisfies(
        &self,
        actions: &[RolePermissionAction],
        data_actions: &[RolePermissionAction],
    ) -> bool {
        for permission in &self.permissions {
            if permission.satisfies(actions, data_actions) {
                return true;
            }
        }
        false
    }
}

impl AsScope for RoleDefinition {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &RoleDefinition {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for RoleDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(&self.id.short_form())?;
        f.write_str(")")?;
        Ok(())
    }
}
impl From<RoleDefinition> for HclImportBlock {
    fn from(role_definition: RoleDefinition) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: role_definition.id.expanded_form(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRmResourceBlockKind::RoleDefinition,
                name: format!(
                    "{}__{}",
                    role_definition.display_name,
                    role_definition.id.short_form()
                )
                .sanitize(),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::ROLE_DEFINITION_ID_PREFIX;
    use eyre::Result;
    use uuid::Uuid;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = format!("{}{}", ROLE_DEFINITION_ID_PREFIX, Uuid::default());
        let id: RoleDefinitionId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id.expanded_form(), expanded);

        Ok(())
    }
}
