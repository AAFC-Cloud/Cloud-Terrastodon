use crate::PrincipalId;
use crate::UnifiedRoleAssignmentId;
use crate::UnifiedRoleDefinitionId;
use crate::tenant_id::AzureTenantId;
use arbitrary::Arbitrary;

/// An Entra role assignment.
///
/// Not to be confused with an Azure RBAC role assignment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct UnifiedRoleAssignment {
    pub directory_scope_id: String,
    pub id: UnifiedRoleAssignmentId,
    pub principal_id: PrincipalId,
    pub principal_organization_id: AzureTenantId,
    pub resource_scope: String,
    pub role_definition_id: UnifiedRoleDefinitionId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_assignment_json_round_trips_through_facet() -> eyre::Result<()> {
        let json = r#"
        {
            "directoryScopeId": "/",
            "id": "role-assignment-id",
            "principalId": "00000000-0000-0000-0000-000000000001",
            "principalOrganizationId": "00000000-0000-0000-0000-000000000002",
            "resourceScope": "/",
            "roleDefinitionId": "00000000-0000-0000-0000-000000000003"
        }
        "#;

        let assignment = facet_json::from_str::<UnifiedRoleAssignment>(json)?;
        assert_eq!(assignment.directory_scope_id, "/");
        let reparsed =
            facet_json::from_str::<UnifiedRoleAssignment>(&facet_json::to_string(&assignment)?)?;
        assert_eq!(assignment, reparsed);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(UnifiedRoleAssignment);
cloud_terrastodon_registry::register_arbitrary!(UnifiedRoleAssignment);
cloud_terrastodon_registry::register_arbitrary!(Vec<UnifiedRoleAssignment>);
