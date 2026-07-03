use arbitrary::Arbitrary;
use crate::PrincipalId;
use crate::RoleAssignmentId;
use crate::RoleDefinitionId;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use cloud_terrastodon_hcl_types::AzureRmResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;

/// An Azure RBAC role assignment.
///
/// Not to be confused with an Entra role assignment.
#[derive(Debug, Eq, PartialEq, Clone, facet::Facet)]
pub struct RoleAssignment {
    pub id: RoleAssignmentId,
    pub scope: ScopeImpl,
    pub role_definition_id: RoleDefinitionId,
    pub principal_id: PrincipalId,
}

impl<'a> Arbitrary<'a> for RoleAssignment {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            id: RoleAssignmentId::arbitrary(u)?,
            scope: ScopeImpl::arbitrary(u)?,
            role_definition_id: RoleDefinitionId::arbitrary(u)?,
            principal_id: PrincipalId::arbitrary(u)?,
        })
    }
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

impl From<RoleAssignment> for HclImportBlock {
    fn from(role_assignment: RoleAssignment) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            // Terraform doesn't like the case variation, https://github.com/hashicorp/terraform-provider-azurerm/issues/26907
            id: role_assignment
                .id
                .expanded_form()
                .replace("/RoleAssignments/", "/roleAssignments/"),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRmResourceBlockKind::RoleAssignment,
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

cloud_terrastodon_registry::register_thing!(RoleAssignment);
cloud_terrastodon_registry::register_arbitrary!(RoleAssignment);

