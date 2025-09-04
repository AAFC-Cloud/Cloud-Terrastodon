use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum GovernanceRoleAssignmentMemberType {
    Group,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum GovernanceRoleAssignmentStatus {
    Provisioned,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GovernanceRoleAssignmentState {
    Active,
    Eligible,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceRoleAssignment {
    pub id: String,
    #[serde(rename = "linkedEligibleRoleAssignmentId")]
    pub linked_eligible_role_assignment_id: Option<String>,
    #[serde(rename = "memberType")]
    pub member_type: GovernanceRoleAssignmentMemberType,
    #[serde(rename = "roleDefinitionId")]
    pub role_definition_id: Uuid,
    #[serde(rename = "startDateTime")]
    pub start_date_time: DateTime<Utc>,
    #[serde(rename = "endDateTime")]
    pub end_date_time: Option<DateTime<Utc>>,
    pub status: GovernanceRoleAssignmentStatus,
    #[serde(rename = "subjectId")]
    pub subject_id: Uuid,
    #[serde(rename="assignmentState")]
    pub assignment_state: GovernanceRoleAssignmentState,
}
