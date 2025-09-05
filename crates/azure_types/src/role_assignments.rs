use crate::prelude::PrincipalId;
use crate::prelude::RoleAssignmentId;
use crate::prelude::RoleDefinitionId;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RoleAssignment {
    pub id: RoleAssignmentId,
    pub scope: ScopeImpl,
    pub role_definition_id: RoleDefinitionId,
    pub principal_id: PrincipalId,
}

// MARK: HasScope
impl AsScope for RoleAssignment {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &RoleAssignment {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl From<RoleAssignment> for HCLImportBlock {
    fn from(role_assignment: RoleAssignment) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            // Terraform doesn't like the case variation, https://github.com/hashicorp/terraform-provider-azurerm/issues/26907
            id: role_assignment
                .id
                .expanded_form()
                .replace("/RoleAssignments/", "/roleAssignments/"),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::RoleAssignment,
                name: role_assignment.id.short_form().sanitize(),
            },
        }
    }
}

impl RoleAssignment {
    pub fn applies_to(&self, scope: &impl Scope) -> bool {
        scope
            .expanded_form()
            .to_lowercase()
            .starts_with(&self.scope.expanded_form().to_lowercase())
    }
}
