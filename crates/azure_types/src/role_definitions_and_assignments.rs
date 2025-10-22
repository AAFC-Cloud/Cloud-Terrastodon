use crate::prelude::RoleAssignment;
use crate::prelude::RoleAssignmentId;
use crate::prelude::RoleDefinition;
use crate::prelude::RoleDefinitionId;
use crate::prelude::RolePermissionAction;
use crate::prelude::Scope;
use eyre::bail;
use std::collections::HashMap;
use uuid::Uuid;

pub struct RoleDefinitionsAndAssignments {
    pub role_definitions: HashMap<RoleDefinitionId, RoleDefinition>,
    pub role_assignments: HashMap<RoleAssignmentId, RoleAssignment>,
}
impl RoleDefinitionsAndAssignments {
    pub fn try_new(
        role_definitions: impl IntoIterator<Item = RoleDefinition>,
        role_assignments: impl IntoIterator<Item = RoleAssignment>,
    ) -> eyre::Result<Self> {
        let role_definitions = role_definitions
            .into_iter()
            .map(|rd| (rd.id.clone(), rd))
            .collect::<HashMap<_, _>>();
        let role_assignments = role_assignments
            .into_iter()
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
                    "Role assignment {} references unknown role definition {}",
                    ra.id.expanded_form(),
                    ra.role_definition_id.expanded_form()
                );
            }
        }
        Ok(rtn)
    }
}
impl std::fmt::Debug for RoleDefinitionsAndAssignments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoleDefinitionsAndAssignments")
            .field("role_definitions_count", &self.role_definitions.len())
            .field("role_assignments_count", &self.role_assignments.len())
            .finish()
    }
}

impl RoleDefinitionsAndAssignments {
    pub fn iter_role_assignments(
        &self,
    ) -> impl Iterator<Item = (&RoleAssignment, &RoleDefinition)> {
        self.role_assignments.values().filter_map(move |ra| {
            self.role_definitions
                .get(&ra.role_definition_id)
                .map(|rd| (ra, rd))
        })
    }
}

pub trait RoleDefinitionsAndAssignmentsIterTools<'a> {
    fn filter_principal(
        self,
        principal_id: &impl AsRef<Uuid>,
    ) -> impl Iterator<Item = (&'a RoleAssignment, &'a RoleDefinition)>;
    fn filter_scope(
        self,
        scope: &impl Scope,
    ) -> impl Iterator<Item = (&'a RoleAssignment, &'a RoleDefinition)>;
    fn filter_satisfying(
        self,
        required_permissions: &[RolePermissionAction],
        required_data_permissions: &[RolePermissionAction],
    ) -> impl Iterator<Item = (&'a RoleAssignment, &'a RoleDefinition)>;
}
impl<'a, T> RoleDefinitionsAndAssignmentsIterTools<'a> for T
where
    T: IntoIterator<Item = (&'a RoleAssignment, &'a RoleDefinition)>,
{
    fn filter_principal(
        self,
        principal_id: &impl AsRef<Uuid>,
    ) -> impl Iterator<Item = (&'a RoleAssignment, &'a RoleDefinition)> {
        let principal_id = principal_id.as_ref();
        self.into_iter()
            .filter(move |(ra, _)| ra.principal_id == principal_id)
    }
    fn filter_scope(
        self,
        scope: &impl Scope,
    ) -> impl Iterator<Item = (&'a RoleAssignment, &'a RoleDefinition)> {
        self.into_iter().filter(move |(ra, _)| ra.applies_to(scope))
    }
    fn filter_satisfying(
        self,
        required_permissions: &[RolePermissionAction],
        required_data_permissions: &[RolePermissionAction],
    ) -> impl Iterator<Item = (&'a RoleAssignment, &'a RoleDefinition)> {
        self.into_iter()
            .filter(move |(_, rd)| rd.satisfies(required_permissions, required_data_permissions))
    }
}
