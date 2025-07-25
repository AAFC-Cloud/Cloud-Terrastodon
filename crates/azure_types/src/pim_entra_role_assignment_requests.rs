use crate::prelude::EligiblePimEntraRoleAssignment;
use crate::prelude::PrincipalId;
use crate::prelude::TenantId;
use crate::prelude::to_iso8601;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignmentRequest {
    #[serde(rename = "roleDefinitionId")]
    role_definition_id: Uuid,
    #[serde(rename = "resourceId")]
    resource_id: Uuid,
    #[serde(rename = "subjectId")]
    subject_id: PrincipalId,
    #[serde(rename = "assignmentState")]
    assignment_state: RoleAssignmentRequestAssignmentState,
    #[serde(rename = "type")]
    kind: RoleAssignmentRequestKind,
    #[serde(rename = "reason")]
    reason: String,
    #[serde(rename = "ticketNumber")]
    ticket_number: String,
    #[serde(rename = "ticketSystem")]
    ticket_system: String,
    #[serde(rename = "schedule")]
    schedule: RoleAssignmentRequestSchedule,
    #[serde(rename = "linkedEligibleRoleAssignmentId")]
    linked_eligible_role_assignment_id: String,
    // #[serde(rename = "scopedResourceId")]
    // scoped_resource_id: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoleAssignmentRequestAssignmentState {
    Active,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoleAssignmentRequestKind {
    UserAdd,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignmentRequestSchedule {
    #[serde(rename = "type")]
    kind: RoleAssignmentRequestScheduleKind,
    duration: iso8601_duration::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoleAssignmentRequestScheduleKind {
    Once,
}

impl RoleAssignmentRequest {
    pub fn new_self_activation(
        principal_id: PrincipalId,
        tenant_id: TenantId,
        role_assignment: &EligiblePimEntraRoleAssignment,
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
