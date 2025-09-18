use crate::prelude::RolePermissionAction;
use crate::prelude::UnifiedRoleAssignment;
use crate::prelude::UnifiedRoleAssignmentId;
use crate::prelude::UnifiedRoleDefinition;
use crate::prelude::UnifiedRoleDefinitionCollection;
use crate::prelude::UnifiedRoleDefinitionId;
use eyre::bail;
use std::collections::HashMap;
use uuid::Uuid;

pub struct UnifiedRoleDefinitionsAndAssignments {
    pub role_definitions: UnifiedRoleDefinitionCollection,
    pub role_assignments: HashMap<UnifiedRoleAssignmentId, UnifiedRoleAssignment>,
}
impl UnifiedRoleDefinitionsAndAssignments {
    pub fn try_new(
        role_definitions: impl IntoIterator<Item = UnifiedRoleDefinition>,
        role_assignments: impl IntoIterator<Item = UnifiedRoleAssignment>,
    ) -> eyre::Result<Self> {
        let mut role_definitions = UnifiedRoleDefinitionCollection::new(role_definitions);
        role_definitions.canonicalize();
        let role_assignments = role_assignments
            .into_iter()
            .filter(|ra: &UnifiedRoleAssignment| {
                // The following roles should not be used. They have been deprecated and will be removed from Microsoft Entra ID in the future.
                // https://learn.microsoft.com/en-us/entra/identity/role-based-access-control/permissions-reference#deprecated-roles
                // We filter them out because entra no longer includes them in the role definitions list, causing validation errors.
                let bad_ids: [UnifiedRoleDefinitionId; _] = [
                    "d65e02d2-0214-4674-8e5d-766fb330e2c0".parse().unwrap() // Email Verified User Creator
                ];
                !bad_ids.contains(&ra.role_definition_id)
            })
            .map(|ra| (ra.id.clone(), ra))
            .collect::<HashMap<_, _>>();
        let rtn = Self {
            role_definitions,
            role_assignments,
        };
        // Validate that all role assignments have a corresponding role definition
        for ra in rtn.role_assignments.values() {
            if !rtn.role_definitions.contains_key(&ra.role_definition_id) {
                bail!(
                    "Role assignment references unknown role definition, {} entries didn't match: {ra:#?}",
                    rtn.role_definitions.len()
                );
            }
        }
        Ok(rtn)
    }
}

impl UnifiedRoleDefinitionsAndAssignments {
    pub fn iter_role_assignments(
        &self,
    ) -> impl Iterator<Item = (&UnifiedRoleAssignment, &UnifiedRoleDefinition)> {
        self.role_assignments.values().filter_map(move |ra| {
            self.role_definitions
                .get(&ra.role_definition_id)
                .map(|rd| (ra, rd))
        })
    }
}

pub trait UnifiedRoleDefinitionsAndAssignmentsIterTools<'a> {
    fn filter_principal(
        self,
        principal_id: &impl AsRef<Uuid>,
    ) -> impl Iterator<Item = (&'a UnifiedRoleAssignment, &'a UnifiedRoleDefinition)>;
    fn filter_satisfying(
        self,
        required_permissions: &[RolePermissionAction],
    ) -> impl Iterator<Item = (&'a UnifiedRoleAssignment, &'a UnifiedRoleDefinition)>;
}
impl<'a, T> UnifiedRoleDefinitionsAndAssignmentsIterTools<'a> for T
where
    T: IntoIterator<Item = (&'a UnifiedRoleAssignment, &'a UnifiedRoleDefinition)>,
{
    fn filter_principal(
        self,
        principal_id: &impl AsRef<Uuid>,
    ) -> impl Iterator<Item = (&'a UnifiedRoleAssignment, &'a UnifiedRoleDefinition)> {
        let principal_id = principal_id.as_ref();
        self.into_iter()
            .filter(move |(ra, _)| ra.principal_id == principal_id)
    }
    fn filter_satisfying(
        self,
        required_permissions: &[RolePermissionAction],
    ) -> impl Iterator<Item = (&'a UnifiedRoleAssignment, &'a UnifiedRoleDefinition)> {
        self.into_iter()
            .filter(move |(_, rd)| rd.satisfies(required_permissions))
    }
}
