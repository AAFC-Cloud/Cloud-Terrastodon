use crate::AzureTenantId;
use crate::GovernanceRoleAssignment;
use crate::PrincipalId;
use crate::to_iso8601;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, facet::Facet)]
pub struct RoleAssignmentRequest {
    #[facet(rename = "roleDefinitionId")]
    pub role_definition_id: Uuid,
    #[facet(rename = "resourceId")]
    pub resource_id: Uuid,
    #[facet(rename = "subjectId")]
    pub subject_id: PrincipalId,
    #[facet(rename = "assignmentState")]
    pub assignment_state: RoleAssignmentRequestAssignmentState,
    #[facet(rename = "type")]
    pub kind: RoleAssignmentRequestKind,
    #[facet(rename = "reason")]
    pub reason: String,
    #[facet(rename = "ticketNumber")]
    pub ticket_number: String,
    #[facet(rename = "ticketSystem")]
    pub ticket_system: String,
    #[facet(rename = "schedule")]
    pub schedule: RoleAssignmentRequestSchedule,
    #[facet(rename = "linkedEligibleRoleAssignmentId")]
    pub linked_eligible_role_assignment_id: String,
    // #[facet(rename = "scopedResourceId")]
    // scoped_resource_id: Value,
}

#[derive(Debug, Clone, facet::Facet)]
#[repr(C)]
pub enum RoleAssignmentRequestAssignmentState {
    Active,
}
#[derive(Debug, Clone, facet::Facet)]
#[repr(C)]
pub enum RoleAssignmentRequestKind {
    UserAdd,
}
#[derive(Debug, Clone, facet::Facet)]
pub struct RoleAssignmentRequestSchedule {
    #[facet(rename = "type")]
    pub kind: RoleAssignmentRequestScheduleKind,
    #[facet(opaque, proxy = crate::IsoDurationProxy)]
    pub duration: iso8601_duration::Duration,
}

#[derive(Debug, Clone, facet::Facet)]
#[repr(C)]
pub enum RoleAssignmentRequestScheduleKind {
    Once,
}

impl RoleAssignmentRequest {
    pub fn new_self_activation(
        principal_id: PrincipalId,
        tenant_id: AzureTenantId,
        role_assignment: &GovernanceRoleAssignment,
        justification: String,
        duration: Duration,
    ) -> Self {
        Self {
            role_definition_id: role_assignment.role_definition_id,
            resource_id: *tenant_id,
            subject_id: principal_id,
            assignment_state: RoleAssignmentRequestAssignmentState::Active,
            kind: RoleAssignmentRequestKind::UserAdd,
            reason: justification,
            ticket_number: "".to_string(),
            ticket_system: "".to_string(),
            schedule: RoleAssignmentRequestSchedule {
                kind: RoleAssignmentRequestScheduleKind::Once,
                duration: to_iso8601(duration),
            },
            linked_eligible_role_assignment_id: role_assignment.id.clone(),
        }
    }
}
